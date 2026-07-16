use crate::core::models::CornettiResult;

/// Minijinja-based template rendering service.
pub struct TemplatesService {
    template_env: minijinja::Environment<'static>,
    templates_conf: crate::templates::confs::TemplatesConf,
}

impl TemplatesService {
    /// Creates a new template service with a filesystem loader pointing to
    /// the configured templates directory.
    pub fn new(templates_conf: crate::templates::confs::TemplatesConf) -> Self {
        let mut env = minijinja::Environment::new();

        env.set_loader(minijinja::path_loader(&templates_conf.templates_directory));

        TemplatesService {
            template_env: env,
            templates_conf,
        }
    }

    /// Returns a reference to the Minijinja environment.
    pub fn env(&self) -> &minijinja::Environment<'static> {
        &self.template_env
    }

    /// Returns the template configuration.
    pub fn get_conf(&self) -> &crate::templates::confs::TemplatesConf {
        &self.templates_conf
    }

    /// Renders a template with the given context.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if the template is not found or rendering fails.
    pub fn render(
        &self,
        template_name: &str,
        context: &std::collections::HashMap<String, minijinja::Value>,
    ) -> CornettiResult<String> {
        Ok(self
            .template_env
            .get_template(template_name)?
            .render(context)?)
    }
}
