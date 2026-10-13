//! **The one door to a GPU in this test binary** (bl-f824).
//!
//! Every [`TestRenderer`] opens a wgpu instance, and on this platform that is
//! a Vulkan instance through the system loader (`libvulkan.so.1`) onto the
//! software rasterizer. The loader is not safe to drive from two threads at
//! once: caught under a debugger, one test's `vkEnumeratePhysicalDevices` was
//! inside `dlclose` — the loader unloading a driver library it had scanned —
//! while another test's `vkSetDebugUtilsObjectNameEXT` was calling through
//! that library, and the whole test binary died of SIGSEGV with no test named.
//! The suite runs its tests on parallel threads, and three of them each held
//! a renderer, so it happened now and then and passed on the re-run.
//!
//! So a renderer is held only under [`GPU`], for its whole life — created,
//! used and dropped while the lock stands — which serializes the loader and
//! nothing else: the harnesses themselves still build and run in parallel.

use std::sync::{Mutex, MutexGuard, PoisonError};

use egui_kittest::Harness;
use egui_kittest::wgpu::TestRenderer;

/// The lock every renderer in this binary is held under.
static GPU: Mutex<()> = Mutex::new(());

/// **A renderer, and the lock it is held under.** The renderer is declared
/// first, so it drops first: the device is torn down while the lock stands.
pub(crate) struct Glass {
    renderer: TestRenderer,
    _held: MutexGuard<'static, ()>,
}

/// **Take the GPU**: wait for whoever holds it, then open a renderer. A test
/// that panicked holding it poisoned nothing worth refusing over — the lock
/// guards the loader's calls, not a value.
pub(crate) fn glass() -> Glass {
    let held = GPU.lock().unwrap_or_else(PoisonError::into_inner);
    Glass {
        renderer: TestRenderer::new(),
        _held: held,
    }
}

impl Glass {
    /// One frame of `harness`, as the pixels it reached the glass as.
    pub(crate) fn render<State>(&self, harness: &Harness<'_, State>) -> image::RgbaImage {
        self.renderer.render(harness)
    }
}
