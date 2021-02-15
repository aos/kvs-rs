use structopt::StructOpt;
use kvs::Result;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs")]
enum KvsOpts {
    Set {
        #[structopt(index = 1, required = true)]
        key: String,
        #[structopt(index = 2, required = true)]
        value: String,
    },
    Get {
        #[structopt(required = true)]
        key: String,
    },
    Rm {
        #[structopt(required = true)]
        key: String,
    }
}

fn main() -> Result<()> {
    match KvsOpts::from_args() {
        KvsOpts::Get { key: _ } => eprintln!("unimplemented"),
        KvsOpts::Set { key: _, value: _ } => eprintln!("unimplemented"),
        KvsOpts::Rm { key: _ } => eprintln!("unimplemented"),
    }

    std::process::exit(1);
}
