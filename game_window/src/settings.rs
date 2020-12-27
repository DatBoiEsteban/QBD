pub struct Settings {
    graphics: GraphicsSettings,
}

impl Settings {
    pub fn new() -> Settings {
        Self {
            graphics: GraphicsSettings::new(),
        }
    }

    pub fn load() -> Settings {
        // TODO: Must search for all stored settings, if not found generate new settings
        Settings::new()
    }

    pub fn graphics(&self) -> &GraphicsSettings {
        &self.graphics
    }
}

pub struct GraphicsSettings {
    window_size: [u16; 2],
    maximized: bool,
}

impl GraphicsSettings {
    pub fn new() -> GraphicsSettings {
        Self {
            window_size: [1280, 720],
            maximized: false,
        }
    }

    pub fn load() -> GraphicsSettings {
        // TODO: Must search for stored settings, if not found generate a new file
        GraphicsSettings::new()
    }

    pub fn window_size(&self) -> [u16; 2] {
        self.window_size
    }

    pub fn maximized(&self) -> bool {
        self.maximized
    }
}
