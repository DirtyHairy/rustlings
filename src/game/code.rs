use anyhow::Result;

const SECRET: &[u8] = b"AJHLDHBBCJ";

pub fn code_for_level(index: usize, percentage: Option<usize>, salt: Option<u8>) -> Result<String> {
    let index = (index & 0xff) as u8;
    let percentage = percentage.unwrap_or(100) as u8;
    let salt = salt.unwrap_or(0);

    let mut code = [0u8; 10];

    code[0] = ((index & 0x01) << 3) | SECRET[0] | ((salt & 0x01) << 1) | ((percentage & 0x01) << 2);
    code[1] = ((index & 0x02) << 1) | SECRET[1] | ((percentage & 0x02) >> 1);
    code[2] = (index & 0x04) | SECRET[2] | ((percentage & 0x04) >> 1) | ((salt & 0x02) >> 1);
    code[3] = ((index & 0x08) >> 3) | SECRET[3] | ((percentage & 0x08) >> 2);
    code[4] = ((index & 0x10) >> 3) | SECRET[4] | ((percentage & 0x10) >> 1) | ((salt & 0x04) >> 2);

    code[5] = ((index & 0x20) >> 5) | SECRET[5] | ((percentage & 0x20) >> 3) | ((salt & 0x08) >> 2);
    code[6] = ((index & 0xc0) >> 4) | SECRET[6] | ((percentage & 0x40) >> 6);
    code[7] = (index & 0xf).wrapping_add(SECRET[7]);
    code[8] = (index >> 4).wrapping_add(SECRET[8]);

    code[9] = (code.iter().copied().fold(0u8, u8::wrapping_add) & 0xf).wrapping_add(SECRET[9]);

    code[..7].rotate_right((8 - (index & 0x07) as usize) % 7);

    Ok(str::from_utf8(&code)?.to_owned())
}

#[cfg(test)]
mod test {
    use crate::code::code_for_level;

    #[test]
    fn code_0() {
        assert_eq!(code_for_level(0, None, None).unwrap(), "CAJJLDLBCS");
    }

    #[test]
    fn code_46() {
        assert_eq!(code_for_level(46, None, None).unwrap(), "MCANNMDPEM");
    }

    #[test]
    fn code_116() {
        assert_eq!(code_for_level(116, None, None).unwrap(), "LFMGAJNFJY");
    }

    #[test]
    fn code_0_100_0() {
        assert_eq!(code_for_level(0, Some(100), Some(0)).unwrap(), "CAJJLDLBCS");
    }

    #[test]
    fn code_46_100_0() {
        assert_eq!(
            code_for_level(46, Some(100), Some(0)).unwrap(),
            "MCANNMDPEM"
        );
    }

    #[test]
    fn code_116_100_0() {
        assert_eq!(
            code_for_level(116, Some(100), Some(0)).unwrap(),
            "LFMGAJNFJY"
        );
    }

    #[test]
    fn code_0_42_0xb() {
        assert_eq!(
            code_for_level(0, Some(42), Some(0xb)).unwrap(),
            "BCKINDNBCX"
        );
    }

    #[test]
    fn code_46_42_0xb() {
        assert_eq!(
            code_for_level(46, Some(42), Some(0xb)).unwrap(),
            "OBCOMODPER"
        );
    }

    #[test]
    fn code_116_42_0xb() {
        assert_eq!(
            code_for_level(116, Some(42), Some(0xb)).unwrap(),
            "NFOFCKMFJN"
        );
    }
}
