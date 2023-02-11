use std::fmt::Display;

#[derive(Default, Debug, Clone, PartialEq)]
#[repr(C)]
pub struct FieldInfo {
    pub report_type: u32,
    pub report_id: u32,
    pub field_index: u32,
    pub maxusage: u32,
    pub flags: u32,
    pub physical: u32,
    /// physical usage for this field
    pub logical: u32,
    /// logical usage for this field
    pub application: u32,
    /// application usage for this field
    pub logical_minimum: i32,
    pub logical_maximum: i32,
    pub physical_minimum: i32,
    pub physical_maximum: i32,
    pub unit_exponent: u32,
    pub unit: u32,
}

impl Display for FieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ report_type: {:#x}, report_id: {:#x}, field_index: {}, maxusage: {}, flags: {:#x}, physical: {}, logical: {}, application: {:#x}, logical_minimum: {}, logical_maximum: {}, physical_minimum: {}, physical_maximum: {}, unit_exponent: {}, unit: {} }}",
            self.report_type, self.report_id, self.field_index, self.maxusage, self.flags, self.physical, self.logical, self.application, self.logical_minimum, self.logical_maximum, self.physical_minimum, self.physical_maximum, self.unit_exponent, self.unit
        )
    }
}

#[cfg(test)]
mod test {
    use crate::FieldInfo;

    #[test]
    fn field_info() {

        let s = format!("F {} S", FieldInfo {
            application: 1,
            field_index: 2,
            flags: 3,
            logical: 4,
            logical_maximum: 5,
            logical_minimum: 4,
            maxusage: 6,
            physical: 7,
            physical_maximum: 8,
            physical_minimum: 7,
            report_id: 9,
            report_type: 10,
            unit: 11,
            unit_exponent: 12,
        });
        assert_eq!(s, "F { report_type: 0xa, report_id: 0x9, field_index: 2, maxusage: 6, flags: 0x3, physical: 7, logical: 4, application: 0x1, logical_minimum: 4, logical_maximum: 5, physical_minimum: 7, physical_maximum: 8, unit_exponent: 12, unit: 11 } S");
    }
}
