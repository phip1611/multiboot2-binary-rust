use arrayvec::ArrayString;
use core::fmt::Debug;
use core::str::FromStr;
use uefi::prelude::{Boot, SystemTable};
use uefi::table::Revision;
use x86::cpuid::{CpuId, ExtendedFeatures, FeatureInfo, HypervisorInfo};

#[derive(Debug)]
pub struct SysInfo {
    os_name: &'static str,
    cpu_info: CpuInfo,
    uefi_info: UefiInfo,
}

#[allow(unused)]
impl SysInfo {
    pub fn new(uefi_st: &SystemTable<Boot>, cpuid: &CpuId) -> Self {
        Self {
            os_name: "Phips' Rust Kernel",
            cpu_info: CpuInfo::new(&cpuid),
            uefi_info: UefiInfo::new(uefi_st),
        }
    }

    pub fn os_name(&self) -> &'static str {
        self.os_name
    }
    pub fn cpu_info(&self) -> &CpuInfo {
        &self.cpu_info
    }
    pub fn uefi_info(&self) -> &UefiInfo {
        &self.uefi_info
    }
}

#[derive(Debug)]
pub struct UefiInfo {
    firmware_revision: Revision,
    firmware_vendor: ArrayString<128>,
    uefi_revision: Revision,
}

#[allow(unused)]
impl UefiInfo {
    fn new(uefi_st: &SystemTable<Boot>) -> Self {
        let mut firmware_vendor = ArrayString::new();
        uefi_st
            .firmware_vendor()
            .iter()
            .take(firmware_vendor.capacity())
            .map(|c16| char::from(*c16))
            .for_each(|c| firmware_vendor.push(c));
        Self {
            firmware_vendor,
            firmware_revision: uefi_st.firmware_revision(),
            uefi_revision: uefi_st.uefi_revision(),
        }
    }

    pub fn firmware_revision(&self) -> Revision {
        self.firmware_revision
    }
    pub fn firmware_vendor(&self) -> ArrayString<128> {
        self.firmware_vendor
    }
    pub fn uefi_revision(&self) -> Revision {
        self.uefi_revision
    }
}

#[derive(Debug)]
pub struct CpuInfo {
    min_fr_mhz: u16,
    max_fr_mhz: u16,
    features: Option<FeatureInfo>,
    extended_features: Option<ExtendedFeatures>,
    hypervisor_info: Option<HypervisorInfo>,
    extended_brand_string: ArrayString<128>,
    // description for L3, L2, L1D and L1I
    cache_descriptions: [Option<ArrayString<128>>; 4],
}

#[allow(unused)]
impl CpuInfo {
    fn new(cpuid: &CpuId) -> Self {
        let brand_string = cpuid.get_processor_brand_string().unwrap();

        let mut cache_descriptions = [None; 4];
        if let Some(i) = cpuid.get_cache_info() {
            i.into_iter()
                .enumerate()
                .take(4)
                .map(|(index, ci)| (index, ci.desc()))
                .for_each(|(i, desc)| {
                    cache_descriptions[i] = Some(
                        ArrayString::from_str(desc)
                            .unwrap_or(ArrayString::from_str("<unknown>").unwrap()),
                    )
                });
        }

        Self {
            min_fr_mhz: cpuid
                .get_processor_frequency_info()
                .map(|x| x.processor_base_frequency())
                .unwrap_or(0),
            max_fr_mhz: cpuid
                .get_processor_frequency_info()
                .map(|x| x.processor_max_frequency())
                .unwrap_or(0),
            features: cpuid.get_feature_info(),
            extended_features: cpuid.get_extended_feature_info(),
            hypervisor_info: cpuid.get_hypervisor_info(),
            extended_brand_string: ArrayString::from_str(brand_string.as_str()).unwrap_or_default(),
            cache_descriptions,
        }
    }

    pub fn min_fr_mhz(&self) -> u16 {
        self.min_fr_mhz
    }
    pub fn max_fr_mhz(&self) -> u16 {
        self.max_fr_mhz
    }
    pub fn features(&self) -> Option<&FeatureInfo> {
        self.features.as_ref()
    }
    pub fn extended_features(&self) -> Option<&ExtendedFeatures> {
        self.extended_features.as_ref()
    }
    pub fn hypervisor_info(&self) -> &Option<HypervisorInfo> {
        &self.hypervisor_info
    }
    pub fn extended_brand_string(&self) -> ArrayString<128> {
        self.extended_brand_string
    }
    pub fn cache_descriptions(&self) -> [Option<ArrayString<128>>; 4] {
        self.cache_descriptions
    }
}
