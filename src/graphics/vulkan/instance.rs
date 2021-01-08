use std::boxed::Box;
use std::result::Result;
use std::sync::Arc;

use vulkano::instance::{Instance, InstanceExtensions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceLayers {
    pub khronos_validation: bool,
    pub mesa_device_select: bool,
    pub mesa_overlay: bool,
    pub _unbuildable: Unbuildable,
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Unbuildable(());

const VK_LAYER_KHRONOS_VALIDATION: &str = "VK_LAYER_KHRONOS_validation";
const VK_LAYER_MESA_DEVICE_SELECT: &str = "VK_LAYER_MESA_device_select";
const VK_LAYER_MESA_OVERLAY: &str = "VK_LAYER_MESA_overlay";

impl InstanceLayers {
    pub fn none() -> Self {
        Self {
            khronos_validation: false,
            mesa_device_select: false,
            mesa_overlay: false,
            _unbuildable: Unbuildable(())
        }
    }

    fn available() -> Self {
        use vulkano::instance::layers_list;

        let mut layers = Self::none();
        for layer in layers_list().unwrap() {
            let name = layer.name();
            if name == VK_LAYER_KHRONOS_VALIDATION {
                layers.khronos_validation = true;
            } else if name == VK_LAYER_MESA_DEVICE_SELECT {
                layers.mesa_device_select = true;
            } else if name == VK_LAYER_MESA_OVERLAY {
                layers.mesa_overlay = true;
            }
        }

        layers
    }

    fn required() -> Self {
        Self::none()
    }

    fn into_names(&self) -> Box<[&'static str]> {
        let mut names = Vec::new();

        if self.khronos_validation {
            names.push(VK_LAYER_KHRONOS_VALIDATION);
        }
        if self.mesa_device_select {
            names.push(VK_LAYER_MESA_DEVICE_SELECT);
        }
        if self.mesa_overlay {
            names.push(VK_LAYER_MESA_OVERLAY);
        }

        names.into_boxed_slice()
    }

    fn difference(&self, other: &Self) -> Self {
        Self {
            khronos_validation:
            self.khronos_validation && !other.khronos_validation,
            mesa_device_select:
            self.mesa_device_select && !other.mesa_device_select,
            mesa_overlay:
            self.mesa_overlay && !other.mesa_overlay,
            _unbuildable: Unbuildable(())
        }
    }

    fn intersection(&self, other: &Self) -> Self {
        Self {
            khronos_validation:
            self.khronos_validation && other.khronos_validation,
            mesa_device_select:
            self.mesa_device_select && other.mesa_device_select,
            mesa_overlay:
            self.mesa_overlay && other.mesa_overlay,
            _unbuildable: Unbuildable(())
        }
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            khronos_validation:
            self.khronos_validation || other.khronos_validation,
            mesa_device_select:
            self.mesa_device_select || other.mesa_device_select,
            mesa_overlay:
            self.mesa_overlay || other.mesa_overlay,
            _unbuildable: Unbuildable(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceFeatures {
    pub extensions: InstanceExtensions,
    pub layers: InstanceLayers,
}

impl InstanceFeatures {
    pub fn none() -> Self {
        Self {
            extensions: InstanceExtensions::none(),
            layers: InstanceLayers::none(),
        }
    }

    fn available() -> Self {
        let extensions = InstanceExtensions::supported_by_core().unwrap();
        let layers = InstanceLayers::available();
        Self { extensions, layers }
    }

    fn required() -> Self {
        let extensions = vulkano_win::required_extensions();
        let layers = InstanceLayers::required();
        Self { extensions, layers }
    }

    fn difference(&self, other: &Self) -> Self {
        Self {
            extensions: self.extensions.difference(&other.extensions),
            layers: self.layers.difference(&other.layers),
        }
    }

    fn intersection(&self, other: &Self) -> Self {
        Self {
            extensions: self.extensions.intersection(&other.extensions),
            layers: self.layers.intersection(&other.layers),
        }
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            extensions: self.extensions.union(&other.extensions),
            layers: self.layers.union(&other.layers),
        }
    }

    fn superset(&self, other: &Self) -> bool {
        other.difference(self) == Self::none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueriedInstanceFeatures {
    available: InstanceFeatures,
    required: InstanceFeatures,
}

impl QueriedInstanceFeatures {
    pub fn query() -> Result<Self, InstanceFeatures> {
        let available = InstanceFeatures::available();
        let required = InstanceFeatures::required();

        if available.superset(&required) {
            Ok(Self { available, required })
        } else {
            Err(required.difference(&available))
        }
    }

    fn select(&self, requested: &InstanceFeatures) -> InstanceFeatures {
        self.required.union(&self.available.intersection(requested))
    }
}

/// The purpose of the ApplicationInfo struct is to identify the game and engine
/// to the Vulkan driver so that it can apply optimizations
/// specialized to particular game engines.
///
/// This game does not use any pre-existing engine,
/// instead opting for a custom, game-specific engine,
/// and thus is unlikely to ever be recognized by any driver.
/// Nonetheless, the fields are populated with this game's information.
fn application_info<'a>() -> vulkano::instance::ApplicationInfo<'a> {
    use vulkano::instance::ApplicationInfo;
    use vulkano::instance::Version;

    // If you fork this game, you should change the application name,
    // but not the engine name. Perhaps one day a driver will
    // recognize the game, and optimizations for our engine
    // would then be able to benefit your fork as well.
    //
    // Also, if it ever gets to that point, performance-wise,
    // it may be worth spoofing the engine to pretend to be
    // another engine with similar performance characteristics
    // (only after extensive testing, of course).
    //
    // TODO: Automatically retrieve this data from the Cargo.toml or similar.
    ApplicationInfo {
        application_name: Some("Halley Kart".into()),
        application_version: Some(Version { major: 0, minor: 1, patch: 0 }),
        engine_name: Some("Halley Kart".into()),
        engine_version: Some(Version { major: 0, minor: 1, patch: 0 }),
    }
}

/// Create a Vulkan instance with the necessary extensions and layers.
///
/// This function will panic if we fail to create the Vulkan instance,
/// seeing as there's no reasonable way to recover from that.
pub fn create_instance(
    available_features: &QueriedInstanceFeatures,
    requested_features: &InstanceFeatures
) -> Arc<Instance> {
    use vulkano::instance::RawInstanceExtensions;

    let features = available_features.select(&requested_features);

    Instance::new(Some(&application_info()),
                  RawInstanceExtensions::from(&features.extensions),
                  features.layers.into_names().iter().map(|&q| q))
        .expect("Failed to create Vulkan instance.")
}
