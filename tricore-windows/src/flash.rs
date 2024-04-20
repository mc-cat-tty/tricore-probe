use std::process::{Child, Command};

use anyhow::Context;
use tempfile::TempDir;

/// Models an upload of a binary with Memtool.
pub struct MemtoolUpload {
    spawned: Child,
    _temporary_files: TempDir,
}

impl MemtoolUpload {
    /// Uploads a binary to the default device in Memtool.
    ///
    /// It generates a configuration file and uses Memtool's batch functionality to
    /// instruct the program to flash all available sections to the device.
    ///
    /// For the created operation to succeed successfully, a DAS instance must
    /// be already spawned, the device to be flashed is selected based on the given
    /// UDAS port.
    ///
    /// Note that the binary must not contain unflashable sections.
    pub fn start(ihex: String, halt_memtool: bool, udas_port: usize) -> anyhow::Result<Self> {
        let temporary_files =
            TempDir::new().context("Cannot create temporary directory for memtool input")?;

        let input_hex_path = temporary_files.path().join("input.hex");

        std::fs::write(&input_hex_path, ihex)
            .context("Cannot write create temporary input hex file")?;

        let temporary_memtool_config_path = temporary_files.path().join("temp_config.cfg");

        std::fs::write(&temporary_memtool_config_path, create_cfg(udas_port))
            .context("Cannot write create temporary memtool configuration file")?;

        let mtb = if !halt_memtool {
            format!("connect\nopen_file {}\nselect_all_sections\nadd_selected_sections\nprogram\ndisconnect\nexit", input_hex_path.display())
        } else {
            format!(
                "connect\nopen_file {}\n",
                temporary_files.path().join("input.hex").display()
            )
        };

        let batch_file_path = temporary_files.path().join("batch.mtb");

        std::fs::write(&batch_file_path, mtb)
            .context("Cannot create temporary memtool batch file")?;

        let mut process = Command::new(env!("MEMTOOL_PATH")); // MEMTOOL_PATH is checked in the build.rs

        let process = process
            .arg("-c")
            .arg(temporary_memtool_config_path.display().to_string())
            .arg(batch_file_path.display().to_string());
        let spawned = process
            .spawn()
            .with_context(|| "Could not start memtool to flash device")?;
        log::info!("Spawned Infineon Memtool to flash HEX file!");

        Ok(MemtoolUpload {
            spawned,
            _temporary_files: temporary_files,
        })
    }

    /// Waits on the upload process to finish.
    ///
    /// This can take a second, but if the tool fails execution it will hang here.
    /// This can happen when the flash layout is broken or when another debugger
    /// is already attached. The problem can only really be debugged with the GUI
    /// or solved by implementing reading the logs from Memtool.
    pub fn wait(&mut self) {
        let output = self
            .spawned
            .wait()
            .expect("Memtool did not exit with success");
        assert!(output.success());
        log::info!("Infineon Memtool terminated successfully");
    }
}

/// Creates a Memtool configuration.
///
/// The configuration file is templated based on the default configuration in Memtool
/// from the TC37xA family, but the DAS port can be selected freely.
fn create_cfg(udas_port: usize) -> String {
    format!(
"[Main]
Signature=UDE_TARGINFO_2.0
MCUs=Controller0
Description=Triboard with TC39x B-Step (DAS)
Description1=Init TLF35584 C-Step on connect
Description2=switch off FLASH error traps
Architecture=TriCore Aurix2G
Vendor=Starter Kits (DAS)
Board=

[Controller0]
Family=TriCore
Type=TC39xB
Enabled=1
IntClock=100000
ExtClock=20000

[Controller0.Core0]
Protocol=TC2_JTAG
Enabled=1

[Controller0.Core0.LoadedAddOn]
UDEMemtool=1

[Controller0.Core0.Tc2CoreTargIntf]
PortType=DAS
CommDevSel=
MaxJtagClk=5000
DasTryStartSrv=1
DasSrvPath=servers\\udas\\udas.exe
ConnOption=Reset
DiswdtOnReset=1
ExecInitCmds=1
TargetPort=Default
CheckJtagId=1
ScanJTAG=0
Ocds1ViaPod=0
EtksArbiterMode=None
RefreshJtag=0
RefreshHarr=0
ReenableOcds=1
ReduceJtagClock=0
UseDap=0
DapMode=2PIN
SetDebugEnableAb1DisablePin=0
ResetWaitTime=500
ResetMode=Default
OpenDrainReset=0
ExecOnConnectCmds=0
ExecOnExtRstCmds=0
ResetPulseLen=10
AddResetDelay=0
ExecEmemInitOnReset=0x0
UnlockInterface=0
BootPasswd0=0x0
BootPasswd1=0x0
BootPasswd2=0x0
BootPasswd3=0x0
BootPasswd4=0x0
BootPasswd5=0x0
BootPasswd6=0x0
BootPasswd7=0x0
PasswordFile=
HandleBmiHeader=0
SetAutOkOnConnect=0
DontUseWdtSusp=0
InitCore0RamOnConnect=0
IgnoreFailedHaltAfterResetOnConnect=0
TrySystemResetAfterFailedHardwareReset=0
RunStabilityTestOnConnect=0
RunStabilityTestCycles=10
IgnoreFailedEnableOcdsOnConnect=0
UseLbistAwareConnect=0
TC4InitCore0Ram=0
EnableAutomaticCsrmStart=0
EnableAutomaticCsrmRunControl=0
MaxTry=1
UseDflashAccessFilter=1
DetectResetWhileHalted=1
UseTranslateAddr=1
DownloadToAllRams=0
HaltAfterReset=0
HaltAfterHardwareReset=0
TargetAppHandshakeMode=None
TargetAppHandshakeTimeout=100
TargetAppHandshakeParameter0=0x0
TargetAppHandshakeParameter1=0x0
TargetAppHandshakeParameter2=0x0
TargetAppHandshakeParameter3=0x0
SimioAddr=g_JtagSimioAccess
UseStmForPtm=1
ExecOnStartCmds=0
ExecOnHaltCmds=0
ExecOnHaltCmdsWhileHaltedPeriod=0
UseTriggerToBreak=1
UseTL2OnHalt=1
UseOstateStable=1
AllowJtagResetWhileRunning=1
MaxAccRetry=1
AccRetryDelay=10
DebugResetOnDisconnect=0
ReadPmcsrWhileRunning=1
IvIcacheOnHalt=1
IvPlbOnHalt=1
SuspendSlaveCores=0
FilterMemAcc=1
PtmRefClock=0
DasDllPath=das_api.dll
DasHost=
DasStopSrv=1
DasResetHelperBreakAddr=main
DasResetMode=2
DasRemoveLogFile=0
DasForwardSerNum=0
DasSrvSel=-1
DasPortType=0
DasPortSel=0
DasCmdTimeout=1000
DasWaitAfterConnect=0
DasDisconnectSrv=0
DasApiLogging=0

[Controller0.Core0.Tc2CoreTargIntf.InitScript]
; Init TLF35584 C-Step on connect
SET 0xF0036034  0x11100002
SET 0xF0001E00  0x8
SET 0xF0001E10  0x20003C04
SET 0xF0001E04  0x1
SET 0xF0001E14  0x14000000
SET 0xF0001E24  0x501
SET 0xF0001E48  0x00020000
SET 0xF003AF10  0x98000000
SET 0xF003AF14  0x10980000
SET 0xF003AF40  0x30330333
SET 0xF003AE10  0x10980000
SET 0xF003AE40  0x33333033
WAIT 5
SET 0xF0001E54  0xFFF
SET 0xF0001E60  0x17A10001
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E10  0x21003C04
SET 0xF0001E64 0x8756
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x87DE
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x86AD
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x8625
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x8D27
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x8A01
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x87BE
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x8668
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x877D
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
SET 0xF0001E64 0x8795
WAIT 5
SET 0xF0001E54 0x200
WAIT 5
SET 0xF0001E54 0x400
WAIT 5

; switch off FLASH error traps
set 0xF8801104 0x10000
set 0xF8821104 0x10000
set 0xF8841104 0x10000
set 0xF8861104 0x10000
set 0xF8881104 0x10000
set 0xF88C1104 0x10000
set 0xF8040048 0xC0000000

[Controller0.Core0.Tc2CoreTargIntf.OnStartScript]

[Controller0.Core0.Tc2CoreTargIntf.OnHaltScript]

[Controller0.Core0.Tc2CoreTargIntf.Suspend]
STM0=1
STM1=1
STM2=1
STM3=1
STM4=1
STM5=1

[Controller0.PFLASH]
Enabled=1
EnableMemtoolByDefault=1

[Controller0.DF_EEPROM]
Enabled=1
EnableMemtoolByDefault=1

[Controller0.DF_UCBS]
Enabled=1
EnableMemtoolByDefault=1


[Controller0.Core0.Tc2CoreTargIntf.OnConnectScript]"
    )
}
