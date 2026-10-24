use snap_herb_models::ReportRow;

pub const ROWS_PER_PAGE: usize = 40;

#[derive(Debug, Clone, PartialEq)]
pub struct PageRow {
    pub index: usize,
    pub row: ReportRow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetPage {
    pub number: usize,
    pub count: usize,
    pub is_last: bool,
    pub rows: Vec<PageRow>,
}

pub fn paginate_rows(rows: &[ReportRow], rows_per_page: usize) -> Vec<SheetPage> {
    if rows_per_page == 0 || rows.is_empty() {
        return Vec::new();
    }
    let chunks: Vec<Vec<PageRow>> = rows
        .chunks(rows_per_page)
        .enumerate()
        .map(|(page_index, chunk)| {
            chunk
                .iter()
                .enumerate()
                .map(|(offset, row)| PageRow {
                    index: page_index * rows_per_page + offset,
                    row: row.clone(),
                })
                .collect()
        })
        .collect();
    let count = chunks.len();
    chunks
        .into_iter()
        .enumerate()
        .map(|(index, rows)| SheetPage {
            number: index + 1,
            count,
            is_last: index + 1 == count,
            rows,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(icode: &str) -> ReportRow {
        ReportRow {
            icode: icode.into(),
            name: format!("herb {icode}"),
            strength_text: String::new(),
            usage_unit: "เม็ด".into(),
            pack_unit: "กล่อง".into(),
            units_per_pack: 100.0,
            max_packs: 30.0,
            usage: 1.0,
            requested: 1.0,
            added: false,
            reason: String::new(),
        }
    }

    #[test]
    fn pagination_splits_at_forty_rows_and_keeps_global_indexes() {
        let rows: Vec<ReportRow> = (0..41).map(|i| row(&format!("{i}"))).collect();
        let pages = paginate_rows(&rows, ROWS_PER_PAGE);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].rows.len(), 40);
        assert_eq!(pages[1].rows.len(), 1);
        assert_eq!(pages[1].rows[0].index, 40);
        assert!(pages[1].is_last);
    }

    #[test]
    fn empty_input_and_zero_page_size_produce_no_pages() {
        assert!(paginate_rows(&[], ROWS_PER_PAGE).is_empty());
        assert!(paginate_rows(&[row("1")], 0).is_empty());
    }
}
