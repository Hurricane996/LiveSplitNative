use core::fmt;
use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
};

use livesplit_core::{
    HotkeySystem, Layout, Run, Segment, SharedTimer, Timer, TimerPhase,
    layout::{self, LayoutSettings, LayoutState},
    rendering::software::Renderer,
    run::parser::composite,
};
use thiserror::Error;

pub struct LivesplitState {
    pub renderer: Renderer,
    layout: Layout,
    pub(crate) timer: SharedTimer,
    layout_state: LayoutState,
    pub hks: HotkeySystem,

    last_rendered_width: u32,
    last_rendered_height: u32,
}

impl LivesplitState {
    pub fn view(&self) -> Vec<u8> {
        self.renderer.image_data().to_vec()
    }

    // necessary because otherwise we will get ugly tearing during resize
    pub const fn last_rendered_size(&self) -> (u32, u32) {
        (self.last_rendered_width, self.last_rendered_height)
    }

    pub fn update(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.layout.update_state(
            &mut self.layout_state,
            // SAFETY: there are only two threads in this program - ours and the hotkey system.
            // Theoretically the hotkey system *could* panic mid-run, but that is
            // so unlikely that I don't care to recover from it.
            &self.timer.read().expect("Timer lock poisoned!").snapshot(),
        );

        self.last_rendered_width = width;
        self.last_rendered_height = height;
        self.renderer.render(&self.layout_state, [width, height]);
    }

    pub fn load_splits(&mut self, path: &Path) -> Result<(), LoadSplitsError> {
        let mut timer = self.timer.write().expect("Timer lock poisoned!");
        if !timer.current_phase().is_running() && !timer.current_phase().is_paused() {
            let run_bytes = fs::read(path)?;
            let run = composite::parse_and_fix(&run_bytes, path.parent())
                .map_err(|_| LoadSplitsError::ParseError)?;
            timer
                .replace_run(run.run, true)
                .map_err(|_| LoadSplitsError::ParseError)?;
        }

        Ok(())
    }

    pub fn save_splits(&self, path: &Path) -> Result<(), SaveSplitsError> {
        let mut timer = self.timer.write().expect("Timer lock poisoned!");

        timer.mark_as_unmodified();

        // this is stupid but sadly necessary because of bad API design
        let mut s = String::new();

        livesplit_core::run::saver::livesplit::save_timer(&timer, &mut s)?;

        drop(timer);

        File::create(path)?.write_all(s.as_bytes())?;

        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.timer
            .read()
            .expect("Timer lock poisoned!")
            .run()
            .has_been_modified()
    }

    pub fn load_layout(&mut self, path: &std::path::Path) -> Result<(), LoadLayoutError> {
        let mut reader = BufReader::new(File::open(path)?);
        let settings = LayoutSettings::from_json(&mut reader);

        let layout = if let Ok(settings) = settings {
            Layout::from_settings(settings)
        } else {
            reader.seek(SeekFrom::Start(0))?;
            let mut buf = String::new();
            reader.read_to_string(&mut buf)?;

            layout::parser::parse(&buf).map_err(|_| LoadLayoutError::ParseError)?
        };

        self.layout = layout;

        Ok(())
    }

    pub fn is_timer_mid_run(&self) -> bool {
        matches!(
            self.timer
                .read()
                .expect("Timer lock poisoned!")
                .current_phase(),
            TimerPhase::Running | TimerPhase::Paused
        )
    }

    pub fn with_settings(settings: &crate::app_settings::Settings) -> Self {
        let run = {
            let mut run = Run::new();

            run.set_game_name("Game");
            run.set_category_name("Category");
            run.push_segment(Segment::new("Time"));

            run
        };

        // SAFETY: we know this is safe because the run is guaranteed valid - we constructed it ourselves
        let timer = Timer::new(run).unwrap();

        let mut layout = Layout::default_layout();

        let layout_state = layout.state(&timer.snapshot());
        let timer = timer.into_shared();

        // The hotkey system can fail to initialize if multiple keys are set the same in the configuration.
        // Unfortunately, as the current system is designed upstream, we have no platform-independent way to
        // detect this and tell the difference between that and an actual panic-worthy failure.
        // Fortunately, in order for this to happen the user would have to manually edit the config file, and
        // if they are doing that they are living in a state of sin and deserve what happens to them
        let hks = HotkeySystem::with_config(timer.clone(), settings.hkc)
            .expect("Failed to initialize hotkey system");

        let mut me = Self {
            renderer: Renderer::new(),
            layout,
            timer,
            layout_state,
            hks,

            last_rendered_width: 0,
            last_rendered_height: 0,
        };

        me.renderer.render(&me.layout_state, [1920, 1080]);

        settings.layout_path.as_ref().inspect(|path| {
            me.load_layout(path).ok();
            me.layout_state = me
                .layout
                .state(&me.timer.read().expect("Timer lock poisoned!").snapshot());
        });

        settings.splits_path.as_ref().inspect(|path| {
            //
            me.load_splits(path).ok();
        });

        me
    }

    pub const fn save_hotkeys_to_settings(&self, app_settings: &mut crate::app_settings::Settings) {
        app_settings.hkc = self.hks.config();
    }

    pub fn disable_hotkeys(&mut self) -> std::result::Result<(), livesplit_core::hotkey::Error> {
        self.hks.deactivate()
    }

    pub fn enable_hotkeys(&mut self) -> std::result::Result<(), livesplit_core::hotkey::Error> {
        self.hks.activate()
    }
}

#[derive(Error, Debug)]
pub enum LoadSplitsError {
    #[error("Failed to load run")]
    IoError(#[from] io::Error),
    #[error("Failed to parse run")]
    ParseError,
}

#[derive(Error, Debug)]
pub enum SaveSplitsError {
    #[error("Failed to serialize run")]
    IoError(#[from] io::Error),
    #[error("Failed to write run")]
    ParseError(#[from] fmt::Error),
}

#[derive(Error, Debug)]
pub enum LoadLayoutError {
    #[error("Failed to load layout")]
    IoError(#[from] io::Error),
    #[error("Failed to parse layout")]
    ParseError,
}
