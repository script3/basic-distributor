# Basic Distributor

This contract implements a basic distribution method using temporary storage for smaller distributions. If you are intending to distribute to a large amount of users, this contract will be expensive to use.

However, for distributions with ~ 1k users or less, this contract provides a simple alternative to a Merkle Proof distribution method. This contract can support larger distributions, but it will become expensive to initialize.

## Safety

Basic Distributor has not had an audit conducted. If an audit is conducted, it will appear here.

Basic Distributor is made available under the MIT License, which disclaims all warranties in relation to the project and which limits the liability of those that contribute and maintain the project, including Script3. You acknowledge that you are solely responsible for any use of the Basic Distributor and you assume all risks associated with any such use.
