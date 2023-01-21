# Proof Bundling in EOSIO/Antelope Transactions
The [Halo 2](https://halo2.dev/) proving system comes with a powerful feature called [Recursive Proof Composition](https://eprint.iacr.org/2019/1021). It allows for multiple zero knowledge proofs of the same arithmetic circuit to be bundled together into a single proof bundle. This feature of Halo 2 can be exploited to scale private transactions on EOSIO/Antelope blockchains.

## Groth16 versus Halo2
Some notes on performance: The currently widely adopted Groth16 proving system is heavily optimized in regards to verification time and proof size which is both small and even constant for any zk-SNARK independent of the complexity of the underlying arithmetic circuit. This means that Groth16 proof verification time scales *linearly* with the number of proofs to be verified.

With Halo2's recursive proof composition, however, the verification time of multiple zk-SNARKs bundled together scales *logarithmically* with the number of proofs. While a single proof verification in Halo2 is more expensive than in Groth16, the overall performance of Halo2 can significantly increase and even beat Groth16 in case of multiple proofs being bundled and verified together. This is even true for a single-threaded Halo2 verifier but in contrast to Groth16 the proof verification in Halo2 can even be performed multi-threaded.

For a more detailed explanation of the benefits of Halo2 checkout the [Technical Explainer](https://electriccoin.co/blog/technical-explainer-halo-on-zcash/).
For use cases of complex privacy transactions on EOSIO/Antelope containing multiple zk-SNARKs refer to the [ZEOS Whitepaper](https://github.com/mschoenebeck/zeos-docs/releases/download/v1.0.0/zeos_whitepaper_v1.0.0.pdf) pages 34 to 37.

## Concept for EOSIO/Antelope Transactions
In order to exploit the recursive proof composition feature of Halo2 for EOSIO/Antelope transactions the following concept is introduced.

### The Problem
The ZEOS privacy actions (aka [ZActions](zactions.md)) all depend on a tuple of zero knowledge proof and corresponding public inputs $(\pi_{C_{zeos}, \omega, x}, x)$ in order to be executed. Each zaction is then executed as a seperate EOSIO/Antelope action in which the corresponding proof is verified independently from all other [ZActions](zactions.md) of the same EOSIO/Antelope transaction. In order to take advantage of the recursive proof composition feature of Halo2, the proofs of all [ZActions](zactions.md) of the same EOSIO/Antelope transaction are bundled into a single proof bundle $\Pi$ which then depends on the set of public inputs $X$ of all [ZActions](zactions.md) in that particular transaction.

$$\Pi_{C_{zeos}, \Omega, X} = \sum_{i=1}^{\mathsf{n}} \pi_{C_{zeos}, \omega_i, x_i}$$

where:
- $\sum$ is the composition function to bundle multiple zero knowledge proofs
- $n$ is the number of [ZActions](zactions.md) to be executed
- $\Omega$ is the set of private inputs $(\omega_1, \omega_2, ..., \omega_n)$
- $X$ is the set of public inputs $(x_1, x_2, ..., x_n)$

But if the proofs of all [ZActions](zactions.md) of an EOSIO/Antelope transaction were to be composed into a single proof bundle, there is also only one single EOSIO/Antelope action to verify this proof bundle and thus validates all subsequent [ZActions](zactions.md). But if that is the case, then all subsequent [ZActions](zactions.md) need to be able to *reference* their public inputs from this one EOSIO/Antelope action that verifies the bundle.

**After all, it can't be trusted that the user executes the subsequent [ZActions](zactions.md) with the same public inputs that were used to verify the proof bundle.**

Since EOSIO/Antelope actions are executed independently from each other, there is no (direct) way, to enforce the execution of two independent EOSIO/Antelope actions with the same input parameters. To prevent fraudulent actions (like verifying the proof bundle using one set of public inputs $X$ but then execute the subsequent [ZActions](zactions.md) using a different set of public inputs $Y$) it needs to be *guaranteed* by the ZEOS smart contract that the proof bundle is verified with the very same public inputs that are used to execute the subsequent [ZActions](zactions.md).

The following graphics illustrate the problem. The first picture shows a transaction execution without proof composition where each zaction verifies its own zero knowledge proof $\pi_i$ using the corresponding public inputs $x_i$.
<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/eosio_tx.png?raw=true">

The second picture shows the same transaction using proof composition where the first action verifies the zero knowledge proof bundle $\Pi$ using the valid set of public inputs $X$ but the subsequent [ZActions](zactions.md) are executed using a different, fraudulent set of public inputs $Y$. This way an attacker is able to defraud the system by executing invalid [ZActions](zactions.md).
<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/eosio_tx_fraud.png?raw=true">

### The Solution
It needs to be guaranteed that subsequent [ZActions](zactions.md) which are validated by the proof bundle are executed with the exact same public inputs as the proof bundle was verified with. In order to achieve this, the following concept is introduced.

#### The [exec](proof-bundling/exec.md) action
The ZEOS smart contract features an 'exec' action which takes a sequence of [ZActions](zactions.md) (i.e. a ztransaction) as parameter and executes them one by one. This EOSIO/Antelope action is only executable by the ZEOS smart contract itself, to ensure it is only executed if the proof bundle that validates the sequence of zactions is verified within the same transaction.

#### Third party smart contracts
As described in [Private Deposits & Withdrawals](private-deposits-withdrawals.md) all third party smart contracts which choose to implement private withdrawals depend on executing [ZActions](zactions.md) as EOSIO/Antelope inline actions. To be able to support those third party private withdrawal actions they need to follow certain design guidelines which make them look very similar to the [exec](proof-bundling/exec.md) action of the ZEOS smart contract:

- They take a sequence of [ZActions](zactions.md) as their *first* action parameter
- They are only executable by the ZEOS smart contract
- They need to be whitelisted by the ZEOS smart contract

#### The [begin](proof-bundling/begin.md) and [step](proof-bundling/step.md) actions
Two more actions are introduced as part of the ZEOS smart contract:

- [begin](proof-bundling/begin.md): This action takes a Halo 2 proof bundle and a sequence of [packed](https://developers.eos.io/manuals/eos/v2.0/cleos/command-reference/convert/pack_action_data) EOSIO/Antelope actions. The packed actions within the sequence need to be whitelisted by the ZEOS smart contract, may contain a sequence of [ZActions](zactions.md) as their *first* action parameter and are ready to be executed, after the proof bundle has been verified. *In addition to that the begin action takes a sequence of encrypted UTXO ciphertexts, $\mathsf{notes_{enc}}$, which is neglected here.*
- [step](proof-bundling/step.md): Each 'step' action is a placeholder which executes exactly one of the packed EOSIO/Antelope actions from the sequence of actions passed to 'begin'. It is a wrapper that executes an EOSIO/Antelope action and, if applicable, the 'exec' action right afterwards with the sequence of [ZActions](zactions.md) which is serialized as the first parameter of the preceding EOSIO/Antelope action. The number of 'step' actions in the EOSIO/Antelop transaction must equal the number of EOSIO/Antelope actions in the sequence passed to 'begin'.

The following example is given to illustrate the concept:

<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/eosio_tx_full_example.png?raw=true">