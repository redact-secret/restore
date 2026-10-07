// Unicode 17.0 General_Category=Cf ranges generated with Node 22.23.1.
// Used for parity with the pinned vault MARKER_PATTERN; no normalization.
pub(crate) fn is_format(c: char) -> bool {
    matches!(c as u32,
        0xad |
        0x600..=0x605 |
        0x61c |
        0x6dd |
        0x70f |
        0x890..=0x891 |
        0x8e2 |
        0x180e |
        0x200b..=0x200f |
        0x202a..=0x202e |
        0x2060..=0x2064 |
        0x2066..=0x206f |
        0xfeff |
        0xfff9..=0xfffb |
        0x110bd |
        0x110cd |
        0x13430..=0x1343f |
        0x1bca0..=0x1bca3 |
        0x1d173..=0x1d17a |
        0xe0001 |
        0xe0020..=0xe007f
    )
}
