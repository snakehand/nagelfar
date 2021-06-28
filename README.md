# nagelfar
Exploring some Rust safety aspects in a mock code example

The account type is concidered to be a simple vendor ( merchant account ), with the following actions:

    Deposit: Represent a sale where funds are transferred from a customer to the clients account
    Withdrawal: Is a transfer from the clients account to an external bank account
    Dispute: Represents a customers claim that there was a problem with the sale (Depsoit) - this puts a hold on the associated funds
    Resolve: Rpresents that the dispute was resolved, by for instance the customer recieving a replacement item
    Chargeback: Represents that the dispute was settled by returning funds to the customer, this freezes the account

