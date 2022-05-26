use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

mod dat;
mod dff;
mod ide;
mod ipl;
mod txd;

pub use self::{dat::Dat, dff::Dff, ide::Ide, ipl::Ipl};

pub struct ViceCityPluginGroup;
impl PluginGroup for ViceCityPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(dat::DatPlugin)
            .add(dff::DffPlugin)
            .add(ide::IdePlugin)
            .add(ipl::IplPlugin)
            .add(txd::TxdPlugin);
    }
}
