use plotters::{
    prelude::{
        ChartBuilder, 
        BitMapBackend, 
        PathElement, 
        IntoDrawingArea, 
        DrawingArea,
    }, 
    series::LineSeries, style::{
        WHITE, 
        RED, 
        BLACK, 
        IntoFont, 
        Color,
    }, 
    coord::Shift,
};
struct Chart<'a> {
    root: DrawingArea<BitMapBackend<'a>, Shift>,
}
impl Chart<'_> {
    pub fn new() -> Self {
        let root = BitMapBackend::new("plotters-doc-data/0.png", (640, 480)).into_drawing_area();
        Self { root: root }
    }
    ///
    pub fn plott(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&self.root)
            .caption("y=x^2", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)?;
    
        chart.configure_mesh().draw()?;
    
        chart
            .draw_series(LineSeries::new(
                (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                &RED,
            ))?
            .label("y = x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
    
        // root.present()?;
    
        Ok(())
    }
    ///
    pub fn present(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.root.present()?)
    }
}