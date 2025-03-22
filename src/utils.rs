use core::ops::{Bound, Range, RangeBounds, RangeTo};

#[inline]
#[track_caller]
fn range_overflow(s: &str) -> ! {
    panic!("attempted to index slice {s} maximum usize");
}

/// Checked and convert any range to normal [`Range`]
#[must_use]
#[track_caller]
pub fn range<R>(range: R, bounds: RangeTo<usize>) -> Range<usize>
where R: RangeBounds<usize>,
{
    let len = bounds.end;

    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => {
            start.checked_add(1)
                .unwrap_or_else(|| range_overflow("from after"))
        },
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&end) => {
            end.checked_add(1)
                .unwrap_or_else(|| range_overflow("up to"))
        },
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    assert!(start <= end,
            "slice index starts at {start} but ends at {end}");

    assert!(end <= len,
            "range end index {end} out of range for slice of length {len}");

    Range { start, end }
}
