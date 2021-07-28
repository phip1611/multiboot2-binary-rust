use x86::cpuid::{CpuId, FeatureInfo, ExtendedFeatures, HypervisorInfo};
use core::fmt::{Debug, Formatter};
use uefi::table::{SystemTable, Boot, Revision};
use alloc::string::String;
use uefi::{CStr16, Char16};
use arrayvec::ArrayString;
use core::str::FromStr;

#[derive(Debug)]
pub struct SysInfo {
    os_name: &'static str,
    cpu_info: CpuInfo,
    uefi_info: UefiInfo,
}

impl SysInfo {
    pub fn new(uefi_st: &SystemTable<Boot>, cpuid: &CpuId) -> Self {
        Self {
            os_name: "Phips' Rust Kernel",
            cpu_info: CpuInfo::new(&cpuid),
            uefi_info: UefiInfo::new(uefi_st),
        }
    }
}

#[derive(Debug)]
pub struct UefiInfo {
    firmware_revision: Revision,
    firmware_vendor: ArrayString::<128>,
    uefi_revision: Revision,
}

impl UefiInfo {
    fn new(uefi_st: &SystemTable<Boot>) -> Self {
        let mut firmware_vendor = ArrayString::new();
        uefi_st.firmware_vendor().iter()
            .take(firmware_vendor.capacity())
            .map(|c16| char::from(*c16))
            .for_each(|c| {
                firmware_vendor.push(c)
            });
        Self {
            firmware_vendor,
            firmware_revision: uefi_st.firmware_revision(),
            uefi_revision: uefi_st.uefi_revision(),
        }
    }
}

#[derive(Debug)]
pub struct CpuInfo {
    min_fr_mhz: u16,
    max_fr_mhz: u16,
    features: FeatureInfo,
    extended_features: ExtendedFeatures,
    hypervisor_info: Option<HypervisorInfo>,
    extended_brand_string: ArrayString::<128>,
    // description for L3, L2, L1D and L1I
    cache_descriptions: [Option<ArrayString::<128>>; 4],
}

impl CpuInfo {
    fn new(cpuid: &CpuId) -> Self {
        let extended_info = cpuid.get_extended_function_info().unwrap();
        let brand_string = extended_info.processor_brand_string().unwrap();

        let mut cache_descriptions = [None; 4];
        if let Some(i) = cpuid.get_cache_info() {
            i.into_iter()
                .enumerate()
                .take(4)
                .map(|(index, ci)| (index, ci.desc()))
                .for_each(|(i, desc)| {
                    cache_descriptions[i] = Some(ArrayString::from_str(desc)
                        .unwrap_or(ArrayString::from_str("<unknown>").unwrap()))
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
            features: cpuid.get_feature_info()
                .unwrap_or(FeatureInfo::default()),
            extended_features: cpuid.get_extended_feature_info()
                .unwrap_or(ExtendedFeatures::default()),
            hypervisor_info: cpuid.get_hypervisor_info(),
            extended_brand_string: ArrayString::from_str(brand_string).unwrap_or_default(),
            cache_descriptions
        }
    }
}
