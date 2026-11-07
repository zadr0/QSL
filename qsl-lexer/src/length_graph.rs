//! Compatibility path; implementation is owned by qsl-core.
pub use qsl_core::length_graph::*;

#[cfg(test)]
mod tests {

    use super::*;
    fn graph(src: &str) -> LengthGraph {
        LengthGraph::analyze(&crate::parse_qsl(src).unwrap()[0])
            .unwrap()
            .unwrap()
    }
    #[test]
    fn two_equal_scalars_form_three_events() {
        let p = graph("&P{i32 id;u8? a;u8? b;}");
        assert_eq!(p.tag_bytes, 1);
        for (i, forced, possible) in [(0, 0, 0), (1, 0, 3), (2, 3, 3)] {
            let n = &p.nodes[p.event_index(i).unwrap()];
            assert_eq!((n.forced, n.possible), (forced, possible));
        }
    }
    #[test]
    fn many_equal_scalars_need_no_exponential_state_enumeration() {
        for count in [13, 32, 64] {
            let fields = (0..count).map(|n| format!("u8? p{n};")).collect::<String>();
            let p = graph(&format!("&P[predict_seq,opt_table]{{{fields}}}"));
            assert_eq!(p.nodes.len(), 3);
            assert_eq!(p.metadata_bytes, (count + 7) >> 3);
            assert_eq!(
                p.nodes[2].forced,
                if count == 64 {
                    u64::MAX
                } else {
                    (1 << count) - 1
                }
            );
        }
    }
    #[test]
    fn string_ranges_force_some_fields_without_erasing_ambiguity() {
        let p = graph("&S{u32 id;str?(3..33) name;str?(1..255) description;}");
        for (i, forced, possible) in [
            (0, 0, 0),
            (1, 2, 2),
            (2, 2, 2),
            (3, 0, 3),
            (4, 0, 3),
            (33, 0, 3),
            (34, 2, 3),
            (255, 2, 3),
            (256, 3, 3),
            (288, 3, 3),
        ] {
            let n = &p.nodes[p.event_index(i).unwrap()];
            assert_eq!((n.forced, n.possible), (forced, possible), "I={i}");
        }
        assert_eq!(p.metadata_bytes, 2);
        assert_eq!(p.fields[0].slot, Some((1, 1)));
        assert_eq!(p.fields[1].slot, None);
        assert_eq!(
            p.nodes
                .iter()
                .skip(1)
                .map(|n| n.start - 1)
                .collect::<Vec<_>>(),
            [0, 2, 33, 255]
        );
        for len in 0..=p.max_payload {
            let rank = p
                .nodes
                .iter()
                .skip(1)
                .map(|n| usize::from(len > n.start - 1))
                .sum::<usize>();
            assert_eq!(Some(rank), p.event_index(len));
        }
    }
}
