use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "libvirtd auto balloon service")]
pub struct Opt {
    #[structopt(short = "r", long, default_value = "500")]
    pub guest_reserved: u64,
    #[structopt(short = "R", long, default_value = "500")]
    pub host_reserved: u64,
    #[structopt(short = "p", long, default_value = "0.0")]
    pub reserved_percent: f32,
    #[structopt(short = "a", long, default_value = "64")]
    pub align: u64,
    #[structopt(short = "d", long, default_value = "0.25")]
    pub duration: f32,
    #[structopt(short = "h", long, default_value = "16")]
    pub history_count: usize,
    #[structopt(short = "c", long, default_value = "qemu:///system")]
    pub connection: String,
}
