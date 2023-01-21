# step
This action is just a placeholder for one of the EOSIO/Antelope actions in the sequence of actions passed as a parameter to the preceeding [begin](begin.md) action. It is part of a sequence of 'step' actions succeeding the execution of a [begin](begin.md) action. It is used to execute exactly one of the actions in the *transaction buffer* of the ZEOS smart contract.

## Parameters
None.

## Flow
The following steps specify the flow of 'step'.

### Step 0
The 'step' action is called as part of an EOSIO/Antelope transaction.

### Step 1
Check if the *transaction buffer* of the ZEOS smart contract is initialized. If not, 'begin' hasn't been executed and execution must cancel.

### Step 2
Pop the next EOSIO/Antelope action to be executed from the *transaction buffer* and execute it as an inline action.

### Step 3
If the EOSIO/Antelope action which was just executed in the previous step is *not* the [exec](exec.md) action of the ZEOS smart contract itself, check if it depends on a sequence $\mathsf{ztx}$ of zactions. If so, execute [exec](exec.md) with the sequence of zactions $\mathsf{ztx}$ of the actual EOSIO/Antelope action (executed in [step 2](#step-2)) as an inline action.

The third party smart contract action from [step 2](#step-2) can thus be assured that the sequence of zactions (which was just processed by the third party contract in [step 2](#step-2)) is actually executed *right after the action itself*.

### Step 4
If this was the last action in the *transaction buffer*, reset the buffer.