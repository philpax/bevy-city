use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

mod common;

mod dat;
mod ide;
mod ipl;

pub use dat::{Dat, DatPlugin};
pub use ide::{Ide, IdePlugin};
pub use ipl::{Ipl, IplPlugin};

pub struct ViceCityPluginGroup;
impl PluginGroup for ViceCityPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(DatPlugin).add(IdePlugin).add(IplPlugin);
    }
}
