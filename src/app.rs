pub struct App<'a> {
    normal_cursor: NormalCursor,
    insert_cursor: InsertCursor,
    visual_cursor: VisualCursor,
    current_times: String,
    current_mode: Mode,
    audio_params: OutputDeviceParameters,
    command_buf: String,
    //file_path: String,
    file_name: String,
    theme: HashMap<String, style::Color>,
    x_bound: u16,
    y_bound: u16,
    cols: Vec<Vec<Vec<Span<'a>>>>,
    yank_buf: Vec<Vec<Vec<Span<'a>>>>,
    //constrains: Vec<Constraint>,
    help_page: usize,
    is_help: bool,
    should_leave: bool,
}

impl App<'_> {
    fn count_lines(&mut self) {
        let max_y = self.cols[1..].iter().map(|it| it.len()).max().unwrap_or(0);
        let mut cols = (0..max_y as isize)
            .map(|it| (it - self.normal_cursor.y as isize).abs())
            .map(|it| vec![Span::from(it.to_string()).style(self.theme["fg_dark"])])
            .collect::<Vec<_>>();
        cols[self.normal_cursor.y as usize][0] =
            Span::from(self.normal_cursor.y.to_string()).style(self.theme["orange"]);
        self.cols[0] = cols;
    }
    fn count_bound(&self) -> usize {
        let bound = self.cols[self.normal_cursor.x as usize][self.normal_cursor.y as usize].iter().map(|it| it.content.len() + 1).sum::<usize>();
        if bound < 14 { 14 } else { bound }
    }
}
