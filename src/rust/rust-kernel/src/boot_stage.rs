use crate::logger::{LOGGER};
use utils::fakelock::FakeLock;
use derive_more::Display as DeriveMoreDisplay;
use crate::panic::PANIC_HANDLER;
use crate::kernelalloc::ALLOCATOR;

/// Knows the current [`BootStage`].
pub static BOOT_STAGE: FakeLock<BootStage> = FakeLock::new(BootStage::S0_Initial);

/// During our boot, we experience multiple stages where one stage
/// follows each other. In each state, our system state is different (further developed)
/// and we may need to solve things different or better. For example: As soon as we have
/// UEFI boot services available, we use them for logging. Once boot services are exited,
/// we may have our own display driver which will write errors to the screen.
/// Note: During the first Boot Stages, we only have one single core, the boot processor (BP).
/// Eventually, we will have multiple multiple cores. When this happens, the unprotected
#[derive(Debug, PartialEq, PartialOrd, Copy, Clone, DeriveMoreDisplay)]
#[allow(non_camel_case_types)]
pub enum BootStage {
    /// Only used as initial value for the enum. This is no real stage.
    S0_Initial,
    /// The state right after the multiboot2 handoff.
    S1_MB2Handoff,
    /// The state when we parsed the multiboot2 information structure and got a reference
    /// to the UEFI system table and **UEFI boot services are enabled**.
    S2_UEFIBootServices,
    /// The state when we exited UEFI boot services (and only runtime services) and we are finally
    /// on our own. TODO this will eventually be renamed to fit the use actual state better whats
    /// happening in code
    S3_UEFIRuntimeServices,
}

impl BootStage {
    pub fn enter<T, R>(&self, prepare_actions: T) -> R
    where
        T: Fn() -> R
    {
        let res = prepare_actions();
        self.switch_to_next_state();
        res
    }

    fn switch_to_next_state(&self) {
        let prev_stage = BOOT_STAGE.get_mut();
        match self {
            BootStage::S0_Initial => {
            }
            BootStage::S1_MB2Handoff => {
                if *prev_stage != BootStage::S0_Initial {
                    panic!("BootStage::UEFI can only be activated when we are in BootStage::Mb2Handoff");
                }
                *prev_stage = BootStage::S1_MB2Handoff;
            }
            BootStage::S2_UEFIBootServices => {
                if *prev_stage != BootStage::S1_MB2Handoff {
                    panic!("BootStage::UEFI can only be activated when we are in BootStage::Mb2Handoff");
                }
                *prev_stage = BootStage::S2_UEFIBootServices;
            }
            BootStage::S3_UEFIRuntimeServices => {
                if *prev_stage != BootStage::S2_UEFIBootServices {
                    panic!("BootStage::Standalone can only be activated when we are in BootStage::UEFI");
                }
                *prev_stage = BootStage::S3_UEFIRuntimeServices
            }
        }

        LOGGER.next_boot_stage(*self);
        PANIC_HANDLER.next_boot_stage(*self);
        ALLOCATOR.next_boot_stage(*self);
    }
}

/// All global structs whose logics depends on the current [`BootStage`] stored
/// in [`BOOT_STAGE`], shall implement this trait. When the boot stage is changed,
/// this method is called on these structs.
pub trait BootStageAware {
    /// Tells the object it should change its internal state to cope with the next boot stage.
    fn next_boot_stage(&self, boot_stage: BootStage);

    /// Convenient wrapper to get the currently active boot stage. This is
    /// (if no programming errors were made) the value that was passed to [`next_boot_stage`].
    /// Using this function, the implementations do not need to store the value.
    fn get_boot_stage(&self) -> BootStage {
        *BOOT_STAGE.get()
    }
}

/*/// All entries correspond to their members of [`BootStage`]. This is a "Follower-Struct"
/// and only there in order to be able to use [`BootStage`] more convenient.
// TODO check debug trait for SystemTable
pub enum BootStageData {
    Mb2HandoffData,
    UEFIData(BootStageUefiData),
    StandaloneData(BootStageStandaloneData)
}

impl BootStageData {
    pub fn get_boot_stage_uefi_data(&self) -> &BootStageUefiData {
        match self {
            BootStageData::UEFIData(inner) => {inner}
            _ => { panic!("Wrong enum variant. Expected `BootStageData::UEFIData`") }
        }
    }
    pub fn get_boot_stage_standalone_data(&self) -> &BootStageStandaloneData {
        match self {
            BootStageData::StandaloneData(inner) => {inner}
            _ => { panic!("Wrong enum variant. Expected `BootStageData::StandaloneData`") }
        }
    }
}

/// Payload of [`BootStageData::UEFIData`].
pub struct BootStageUefiData {
    uefi_st: SystemTable<Boot>
}

impl BootStageUefiData {
    pub fn new(uefi_st: SystemTable<Boot>) -> Self {
        Self { uefi_st }
    }


    pub fn uefi_st(&self) -> &SystemTable<Boot> {
        &self.uefi_st
    }
}

/// Payload of [`BootStageData::StandaloneData`].
pub struct BootStageStandaloneData {
    uefi_st: SystemTable<Runtime>
}

impl BootStageStandaloneData {
    pub fn new(uefi_st: SystemTable<Runtime>) -> Self {
        Self { uefi_st }
    }


    pub fn uefi_st(&self) -> &SystemTable<Runtime> {
        &self.uefi_st
    }
}*/
