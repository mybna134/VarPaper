use anyhow::Result;
use wayvid_library::project::ProjectResources;

/// Project-scoped reader supplied by the application. Paths are relative;
/// implementations must validate resource budgets and directory confinement.
pub trait SceneResources {
    fn exists(&self, name: &str) -> Result<bool>;
    fn read(&self, name: &str) -> Result<Vec<u8>>;
}

impl SceneResources for ProjectResources {
    fn exists(&self, name: &str) -> Result<bool> {
        ProjectResources::exists(self, name)
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        ProjectResources::read(self, name)
    }
}
