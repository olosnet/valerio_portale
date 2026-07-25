#[cfg(feature = "templates")]
templates(500, log_level: Error): {
    *template_rendering_error(500, log_level: Error) => "Template rendering error",
},
