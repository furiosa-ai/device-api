use std::{path::Path, time::SystemTime};

use crate::sysfs;

pub mod error {
    use std::io;

    use thiserror::Error;

    pub type PerformanceCounterResult<T> = Result<T, PerformanceCounterError>;

    /// An error that occurred during parsing or retrieving performance counters.
    #[derive(Debug, Error)]
    pub enum PerformanceCounterError {
        #[error("This device is not in use")]
        DeviceNotInUse,
        #[error("Not found PerformanceCounter file.")]
        NotFoundPerformanceCounterFile,
        #[error("Unexpected PerformanceCounter format.")]
        UnexpectedFileFormat,
        #[error("IoError: {cause}")]
        IoError { cause: io::Error },
    }

    impl From<io::Error> for PerformanceCounterError {
        fn from(e: io::Error) -> Self {
            Self::IoError { cause: e }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Utilization {
    npu_utilization: f64,
    computation_ratio: f64,
    io_ratio: f64,
}

impl Utilization {
    fn new(npu_utilization: f64, computation_ratio: f64, io_ratio: f64) -> Self {
        Self {
            npu_utilization,
            computation_ratio,
            io_ratio,
        }
    }

    pub fn npu_utilization(&self) -> f64 {
        self.npu_utilization
    }

    pub fn computation_ratio(&self) -> f64 {
        self.computation_ratio
    }

    pub fn io_ratio(&self) -> f64 {
        self.io_ratio
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerformanceCounter {
    now: SystemTime,
    cycle_count: usize,
    task_execution_cycle: u32,
    tensor_execution_cycle: u32,
}

impl Default for PerformanceCounter {
    fn default() -> Self {
        Self {
            now: SystemTime::now(),
            task_execution_cycle: Default::default(),
            tensor_execution_cycle: Default::default(),
            cycle_count: Default::default(),
        }
    }
}

impl PerformanceCounter {
    /// Read performance counters about specific device.
    pub fn read<P: AsRef<Path>>(
        base_dir: P,
        dev_name: &str,
    ) -> error::PerformanceCounterResult<PerformanceCounter> {
        let path = sysfs::perf_regs::path(base_dir, dev_name);
        std::fs::read_to_string(&path)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    error::PerformanceCounterError::DeviceNotInUse
                }
                std::io::ErrorKind::NotFound => {
                    error::PerformanceCounterError::NotFoundPerformanceCounterFile
                }
                _ => error::PerformanceCounterError::from(err),
            })
            .and_then(Self::parse)
    }

    pub(crate) fn parse(text: String) -> error::PerformanceCounterResult<PerformanceCounter> {
        let mut counter = PerformanceCounter::default();
        let mut cycle_count_low: usize = 0;
        let mut cycle_count_high: usize = 0;

        let mut valid_line_count = 0;

        let lines = text.lines();
        for line in lines {
            let v: Vec<&str> = line.split(": ").collect();
            match v[0] {
                "TaskExecutionTime" => {
                    valid_line_count += 1;
                    counter.task_execution_cycle = Self::hex_to_u32(v[1])
                }
                "TensorExecutionTime" => {
                    valid_line_count += 1;
                    counter.tensor_execution_cycle = Self::hex_to_u32(v[1])
                }
                "CycleCount" => {
                    valid_line_count += 1;
                    cycle_count_low = Self::hex_to_u32(v[1]) as usize
                }
                "CycleCountHigh" => {
                    valid_line_count += 1;
                    cycle_count_high = Self::hex_to_u32(v[1]) as usize
                }
                _ => (),
            }
        }

        if valid_line_count < 4 {
            return Err(error::PerformanceCounterError::UnexpectedFileFormat);
        }

        counter.cycle_count = (cycle_count_high << 32) | cycle_count_low;
        Ok(counter)
    }

    /// Returns cycle count of the device file.
    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    /// Returns task execution cycle count of the device file.
    pub fn task_execution_cycle(&self) -> u32 {
        self.task_execution_cycle
    }

    /// Returns tensor execution cycle count of the device file.
    pub fn tensor_execution_cycle(&self) -> u32 {
        self.tensor_execution_cycle
    }

    /// Returns the difference between two counters.
    pub fn calculate_increased(&self, other: &Self) -> Self {
        let (prev, next) = if self.now < other.now {
            (self, other)
        } else {
            (other, self)
        };

        // when the cycle count is reversed, NPU was restarted.
        if next.cycle_count < prev.cycle_count {
            return *next;
        }

        let elapsed_time = next.now.duration_since(prev.now);
        if elapsed_time.is_err() {
            return *next;
        }

        let elapsed_time = elapsed_time.unwrap().as_nanos() as usize;
        let cycle_gap = next.cycle_count - prev.cycle_count;

        // Unfortunately, it's a naive implementation for now.
        if (cycle_gap as f64 / elapsed_time as f64) < 0.45 {
            return *next;
        }

        let task_cycle_gap =
            Self::safe_u32_subtract(next.task_execution_cycle, prev.task_execution_cycle);
        let tensor_cycle_gap =
            Self::safe_u32_subtract(next.tensor_execution_cycle, prev.tensor_execution_cycle);

        let task_cycle_gap = std::cmp::min(cycle_gap, task_cycle_gap);
        let tensor_cycle_gap = std::cmp::min(task_cycle_gap, tensor_cycle_gap);

        Self {
            now: next.now,
            cycle_count: cycle_gap,
            task_execution_cycle: std::cmp::min(task_cycle_gap, u32::MAX as usize) as u32,
            tensor_execution_cycle: std::cmp::min(tensor_cycle_gap, u32::MAX as usize) as u32,
        }
    }

    /// Returns NPU utilization based on the difference between two counters.
    pub fn calculate_utilization(&self, other: &Self) -> Utilization {
        let diff = self.calculate_increased(other);

        if diff.task_execution_cycle == 0 {
            return Utilization::default();
        }

        let npu_utilization =
            Self::safe_usize_divide(diff.task_execution_cycle as usize, diff.cycle_count);

        // Sometimes the tensor cycle is larger than the task cycle due to observation timing issues. In this case, an upper bound is applied.
        let computation_ratio = Self::safe_usize_divide(
            diff.tensor_execution_cycle as usize,
            diff.task_execution_cycle as usize,
        );
        let io_ratio = 1.0 - computation_ratio;

        Utilization::new(npu_utilization, computation_ratio, io_ratio)
    }

    // Except for "cycle count", the other fields have a u32 type.
    // If the performance counter value exceeds the range of the type, an overflow occurs,
    // and we use this function to compute the difference.
    fn safe_u32_subtract(fst: u32, snd: u32) -> usize {
        if fst >= snd {
            (fst - snd) as usize
        } else {
            ((1 << 32) | fst as usize) - snd as usize
        }
    }

    fn safe_usize_divide(fst: usize, snd: usize) -> f64 {
        if snd == 0 {
            0.0
        } else {
            fst as f64 / snd as f64
        }
    }

    fn hex_to_u32(hex: &str) -> u32 {
        u32::from_str_radix(&hex[2..], 16).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let res = PerformanceCounter::read("../test_data/test-0/sys", "npu0pe0");
        assert!(res.is_ok());
    }

    #[test]
    fn test_parse() {
        let perf_regs = r"ProgramCounter: 0x10b74
InstructionCount: 0x29b33c
TensorExecutionCount: 0x63ec5
BranchCount: 0x3ff52
BranchTakenCount: 0x2ee32
DramToSramCount: 0x1112
SramToDramCount: 0x73
LastBranchTaken: 0x0
LastBranchTakenTarget: 0x10ab0
TaskExecutionTime: 0x26272443
TensorExecutionTime: 0x256254d7
LastTensorExecutionTime: 0xa7
WaitTag: 0x0
SendSubmissionTail: 0x0
SendCompletionHead: 0x0
RecvSubmissionTail: 0x0
RecvCompletionHead: 0x0
DataMemoryControllerTotalDramToSramIssueCount: 0x1112
DataMemoryControllerTotalSramToDramIssueCount: 0x73
DataMemoryControllerTotalDramToSramCompleteCount: 0x1112
DataMemoryControllerTotalSramToDramCompleteCount: 0x73
DataMemoryControllerTotalDramToSramLatency: 0x74e18f7
DataMemoryControllerTotalSramToDramLatency: 0x418a98
DataMemoryControllerLastDramToSramLatency: 0x3742
DataMemoryControllerLastSramToDramLatency: 0x91b8
LoadStoreSubmissionTail: 0x5
LoadStoreSubmissionHead: 0x5
CycleCount: 0x6f22d16d
CycleCountHigh: 0x0
";

        let res = PerformanceCounter::parse(perf_regs.to_string());
        assert!(res.is_ok());

        let perf_counter = res.unwrap();
        assert_eq!(1864552813, perf_counter.cycle_count);
        assert_eq!(640099395, perf_counter.task_execution_cycle);
        assert_eq!(627201239, perf_counter.tensor_execution_cycle);
    }

    #[test]
    fn test_safe_subtract() {
        assert_eq!(1, PerformanceCounter::safe_u32_subtract(2, 1));
        assert_eq!(1, PerformanceCounter::safe_u32_subtract(0, 0xFFFFFFFF)); // first argument 0 means 0x100000000
        assert_eq!(4294967295, PerformanceCounter::safe_u32_subtract(1, 2)); // 4294967295: 0xFFFFFFFF, first argument 1 means 0x100000001
    }

    #[test]
    fn test_diff() {
        let prev_perf_regs = r"ProgramCounter: 0x4c2c
InstructionCount: 0x98616
TensorExecutionCount: 0x16cdf
BranchCount: 0xe964
BranchTakenCount: 0xab06
DramToSramCount: 0x3f0
SramToDramCount: 0x1a
LastBranchTaken: 0x0
LastBranchTakenTarget: 0x4c2c
TaskExecutionTime: 0x8a035bf
TensorExecutionTime: 0x88c229e
LastTensorExecutionTime: 0x533
WaitTag: 0x0
SendSubmissionTail: 0x0
SendCompletionHead: 0x0
RecvSubmissionTail: 0x0
RecvCompletionHead: 0x0
DataMemoryControllerTotalDramToSramIssueCount: 0x3f0
DataMemoryControllerTotalSramToDramIssueCount: 0x1a
DataMemoryControllerTotalDramToSramCompleteCount: 0x3f0
DataMemoryControllerTotalSramToDramCompleteCount: 0x1a
DataMemoryControllerTotalDramToSramLatency: 0x1ac1040
DataMemoryControllerTotalSramToDramLatency: 0xecd9d
DataMemoryControllerLastDramToSramLatency: 0xd23
DataMemoryControllerLastSramToDramLatency: 0x9199
LoadStoreSubmissionTail: 0x2
LoadStoreSubmissionHead: 0x2
CycleCount: 0x35f8bf98
CycleCountHigh: 0x0
";

        let next_perf_regs = r"ProgramCounter: 0x5270
InstructionCount: 0x100f44
TensorExecutionCount: 0x26773
BranchCount: 0x189c7
BranchTakenCount: 0x1209c
DramToSramCount: 0x69d
SramToDramCount: 0x2c
LastBranchTaken: 0x0
LastBranchTakenTarget: 0x52b4
TaskExecutionTime: 0xe98fb61
TensorExecutionTime: 0xe68014a
LastTensorExecutionTime: 0x533
WaitTag: 0x8
SendSubmissionTail: 0x0
SendCompletionHead: 0x0
RecvSubmissionTail: 0x0
RecvCompletionHead: 0x0
DataMemoryControllerTotalDramToSramIssueCount: 0x69d
DataMemoryControllerTotalSramToDramIssueCount: 0x2c
DataMemoryControllerTotalDramToSramCompleteCount: 0x69c
DataMemoryControllerTotalSramToDramCompleteCount: 0x2c
DataMemoryControllerTotalDramToSramLatency: 0x2d0bf26
DataMemoryControllerTotalSramToDramLatency: 0x19111f
DataMemoryControllerLastDramToSramLatency: 0xd5e
DataMemoryControllerLastSramToDramLatency: 0x9242
LoadStoreSubmissionTail: 0x1
LoadStoreSubmissionHead: 0x0
CycleCount: 0x4235c692
CycleCountHigh: 0x0
";

        let res = PerformanceCounter::parse(prev_perf_regs.to_string());
        assert!(res.is_ok());
        let prev = res.unwrap();

        let res = PerformanceCounter::parse(next_perf_regs.to_string());
        assert!(res.is_ok());
        let next = res.unwrap();

        let values = next.calculate_utilization(&prev);
        assert_eq!(4880, (values.npu_utilization() * 10000.0).round() as i64);
        assert_eq!(9811, (values.computation_ratio() * 10000.0).round() as i64);
        assert_eq!(189, (values.io_ratio() * 10000.0).round() as i64);
    }
}
