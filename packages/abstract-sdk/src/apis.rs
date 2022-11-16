use abstract_os::objects::ans_host::AnsHost;

mod ans;
mod applications;
mod execute;
mod ibc;
mod staking;
mod transfer;
mod version_control;


// Api's can be accessed through trait implementation


 
pub trait AbstractNameService<'a> {
    fn ans(&self, ans_host: &AnsHost)-> Ans<'a> {
        Ans { base: self }
    }
}

impl<'a, T> AbstractNameService <'a> for T
    where T: Sized
{}

pub struct Ans <'a> {
    ans_host: &'a AnsHost
}

impl<'a> Ans<'a> {

}
