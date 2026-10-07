/// Inclusive byte/count bounds. Zero permits no items/bytes in that category.
/// These bounds limit engine allocations, not allocations inside an authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub fields: usize,
    pub field_bytes: usize,
    pub input_bytes: usize,
    pub occurrences: usize,
    pub token_bytes: usize,
    pub captures: usize,
    pub context_bytes: usize,
    pub field_output_bytes: usize,
    pub output_bytes: usize,
    /// Maximum total output/input ratio; token-free empty input may yield zero bytes.
    pub expansion_ratio: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            fields: 128,
            field_bytes: 1024 * 1024,
            input_bytes: 4 * 1024 * 1024,
            occurrences: 16_384,
            token_bytes: 128,
            captures: 128,
            context_bytes: 4096,
            field_output_bytes: 4 * 1024 * 1024,
            output_bytes: 16 * 1024 * 1024,
            expansion_ratio: 16,
        }
    }
}
