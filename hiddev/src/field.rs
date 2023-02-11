use hiddev_sys::FieldInfo;

use std::fmt::Display;

#[derive(Default)]
pub struct InfoVec(pub Vec<FieldInfo>);

impl Display for InfoVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return f.write_str("[]");
        }
        f.write_str("[ ")?;
        let mut first = true;
        for fi in self.0.iter() {
            if !first {
                f.write_str(" ,")?;
            }
            fi.fmt(f)?;
            first = false;
        }
        f.write_str(" ]")
    }
}

#[cfg(test)]
mod test {

    use hiddev_sys::FieldInfo;

    use super::InfoVec;

    #[test]
    fn field_infos() {
        let mut fis = InfoVec::default();
        let mut s = format!("F {} S", fis);
        assert_eq!(s, "F [] S");

        fis.0.push(FieldInfo {
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
        s = format!("F {} S", fis);
        assert_eq!(s, "F [ { report_type: 0xa, report_id: 0x9, field_index: 2, maxusage: 6, flags: 0x3, physical: 7, logical: 4, application: 0x1, logical_minimum: 4, logical_maximum: 5, physical_minimum: 7, physical_maximum: 8, unit_exponent: 12, unit: 11 } ] S");
    }
}
