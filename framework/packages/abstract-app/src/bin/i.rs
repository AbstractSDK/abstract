struct Contract<'a> {
    // Some contract fields
    data: &'a str, // Example data field, could be anything with the specified lifetime
}

struct Bank<'a, 'b> {
    contract_ref: &'a mut Contract<'b>,
    // Other Bank fields
}

impl<'a, 'b> Bank<'a, 'b> {
    fn new(contract: &'a mut Contract<'b>) -> Bank<'a, 'b> {
        Bank {
            contract_ref: contract,
            // Initialize other Bank fields
        }
    }

    // Other methods for the Bank
}

fn main() {
    // Creating a data string with a specific lifetime
    let data = "Some data";

    // Creating a Contract instance with a specific lifetime
    let mut contract = Contract { data: &data };

    // Creating Bank instances with mutable references to the Contract
    let mut bank1 = Bank::new(&mut contract);
    let mut bank2 = Bank::new(&mut contract);

    // Use the bank instances or let them go out of scope
}
