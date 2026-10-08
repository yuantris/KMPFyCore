#[derive(Debug, Clone)]
pub struct GraphConfig {
    /// 数学坐标系中的最小 X
    pub x_min: f64,

    /// 数学坐标系中的最大 X
    pub x_max: f64,

    /// 屏幕宽度
    pub pixel_width: u32,

    /// 屏幕高度
    pub pixel_height: u32,

    /// 最大递归细分深度
    pub max_subdivision: u32,

    /// 最小可继续细分的屏幕长度
    pub min_pixel_distance: f64,

    /// 屏幕 Y 方向允许的最大跳变
    pub discontinuity_threshold: f64,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            x_min: -10.0,
            x_max: 10.0,

            pixel_width: 1000,
            pixel_height: 600,

            max_subdivision: 12,

            min_pixel_distance: 0.5,

            discontinuity_threshold: 1.5,
        }
    }
}