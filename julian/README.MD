# This contract is a decentralized marketplace. It is in testing on Juno, designed to be launched on ATOM once permissionless smart contracts arise.

# Juno Create

*Store*

junod tx wasm store julian.wasm --from deploy --fees 1500000ujuno --gas 20000000 -y

hash: 4ACDB1620F5FE25AABE39055055A8143ACA57C6908FD638E116DE22911290F37

code id: 4630

https://www.mintscan.io/juno/tx/4ACDB1620F5FE25AABE39055055A8143ACA57C6908FD638E116DE22911290F37

*Instantiate*

junod tx wasm instantiate 4630 '{}' --label test_dep_for_ham --admin deploy --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: E22B353E829BCABE26798E4BE07FA45161947140BDFAEDC4B46856FB6094CB9E

contract address: juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra

https://www.mintscan.io/juno/wasm/contract/juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra

# Juno Migrate

*Store*

junod tx wasm store julian.wasm --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: 1D4A433128CAD0BC49CF21A75BD836BC075FF9F0755FB6FE98DF0C0A96DCFB91

code id: 4638

https://www.mintscan.io/juno/tx/1D4A433128CAD0BC49CF21A75BD836BC075FF9F0755FB6FE98DF0C0A96DCFB91/?height=23140032

*Migrate* 

junod tx wasm migrate juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra 4638 '{"migrate_msg":{}}' --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: A266305153292B054DA5AE7A6BB113C3EEE81C5C04E8D4C59648DB3473747A1C

https://www.mintscan.io/juno/tx/A266305153292B054DA5AE7A6BB113C3EEE81C5C04E8D4C59648DB3473747A1C?height=23140104

# Query Contract

*Query Listing Count*

junod query wasm contract-state smart juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra '{"listing_count":{}}'

*Query Listing*

junod query wasm contract-state smart juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra '{"listing":{"listing_id": 1}}'

# Juno Migrate Number 2

*Store*

junod tx wasm store julian.wasm --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: 078AFCA4CAA08F378CD35D5880093C0C3D0A91DDC8352C079AF46FFCC2B40592

code id: 4639

https://www.mintscan.io/juno/tx/078AFCA4CAA08F378CD35D5880093C0C3D0A91DDC8352C079AF46FFCC2B40592/?height=23202159

*Migrate*

junod tx wasm migrate juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra 4639 '{"migrate_msg":{}}' --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: DC2EEE22D7FB8CAB9EB7E9C2B31DF8F0C04F962A1E82C0D513AB4AE68644E783

https://www.mintscan.io/juno/tx/DC2EEE22D7FB8CAB9EB7E9C2B31DF8F0C04F962A1E82C0D513AB4AE68644E783/?height=23202211

# Juno Migrate Number 3

*Store*

junod tx wasm store julian.wasm --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: 5B545F5472359A542BB50BBD1F0B47F44CD039A0480DADB19A039BF8EECBA22D

code id: 4673

https://www.mintscan.io/juno/tx/5B545F5472359A542BB50BBD1F0B47F44CD039A0480DADB19A039BF8EECBA22D/?height=23572178

*Migrate*

junod tx wasm migrate juno1fucsaa4mukx86z5sfxm3k3445eh8c4vcpejzu93457wufh4s6zms4qz6ra 4673 '{"migrate_msg":{}}' --from deploy --fees 1500000ujuno --gas 20000000 -y

tx hash: 491C4A1B1198D3B2524CFC810463F3212A6A5A2FC23B809EC446D5A0313717A8

https://www.mintscan.io/juno/tx/491C4A1B1198D3B2524CFC810463F3212A6A5A2FC23B809EC446D5A0313717A8?height=23572254