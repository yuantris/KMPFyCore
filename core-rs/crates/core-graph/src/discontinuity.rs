use crate::config::GraphConfig;
use crate::point::GraphPoint;

pub struct DiscontinuityDetector;

impl DiscontinuityDetector {
    pub fn should_break(
        previous: GraphPoint,
        current: GraphPoint,
        config: &GraphConfig,
    ) -> bool {
        // NaN / Infinity
        if !previous.y.is_finite() || !current.y.is_finite() {
            return true;
        }

        // X 不连续
        if !previous.x.is_finite() || !current.x.is_finite() {
            return true;
        }

        // 屏幕坐标跳变检测
        let y1 = Self::to_screen_y(previous.y, config);
        let y2 = Self::to_screen_y(current.y, config);

        if !y1.is_finite() || !y2.is_finite() {
            return true;
        }

        let delta_y = (y2 - y1).abs();

        if delta_y > config.pixel_height as f64
            * config.discontinuity_threshold
        {
            return true;
        }

        // 防止跨越奇点。
        //
        // 注意：
        // 这里不是简单判断 x == 0，
        // 而是判断相邻采样点是否跨越 0。
        if previous.x < 0.0 && current.x > 0.0 {
            return true;
        }

        if previous.x > 0.0 && current.x < 0.0 {
            return true;
        }

        false
    }

    fn to_screen_y(y: f64, config: &GraphConfig) -> f64 {
        // 当前这里只需要一个稳定的尺度。
        //
        // 实际项目中建议由 GraphViewport 提供
        // math -> screen 的完整转换。
        //
        // 这里先采用 y 本身进行跳变判断。
        y.clamp(
            -(config.pixel_height as f64 * 10.0),
            config.pixel_height as f64 * 10.0,
        )
    }
}