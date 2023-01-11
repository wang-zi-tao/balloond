use std::{
    collections::HashSet,
    thread,
    time::Duration,
};
use domain::DomainDataBase;
use failure::Fallible;
use log::error;
use structopt::StructOpt;
use virt::{
    connect::Connect,
    domain::Domain,
};

mod cli;
mod domain;


fn main() -> Fallible<()> {
    env_logger::init();
    let opt = cli::Opt::from_args();
    let connection = Connect::open(&opt.connection)?;
    let mut db = DomainDataBase::default();
    loop {
        let mut domain_set=HashSet::new();
        let domain_list=connection.list_domains()?;
        for &domain_id in &domain_list {
            if let Ok(domain) = Domain::lookup_by_id(&connection, domain_id) {
                if let Ok(name) = domain.get_name() {
                    domain_set.insert(name.clone());
                    let result = db
                        .records
                        .entry(name)
                        .or_default()
                        .process_domain(domain,domain_list.len(), &opt);
                    if let Err(e) = result {
                        error!("{}", e);
                    }
                }
            }
            db.records.retain(|name,_|domain_set.contains(name));
        }
        thread::sleep(Duration::from_secs_f32(opt.duration))
    }
}
