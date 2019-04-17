use bitflags::bitflags;

bitflags! {
    pub struct Modifier: u32 {
        const None = 1 << 0;
        const CapsLock = 1 << 16;
        const Shift = 1 << 17;
        const Control = 1 << 18;
        const Option = 1 << 19;
        const Command = 1 << 20;
        const NumericPad = 1 << 21;
        const Help = 1 << 22;
        const Function = 1 << 23;
        const DeviceIndependentFlagsMask = 0xffff0000;
    }
}