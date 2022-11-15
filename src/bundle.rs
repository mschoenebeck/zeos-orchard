//! Structs related to bundles of Orchard actions.

use rand::RngCore;
use rustzeos::halo2::{Proof, ProvingKey};
use crate::{
    action::{RawZAction, ZAction, ZA_MINTAUTH, ZA_MINTFT, ZA_MINTNFT, ZA_BURNAUTH},
    circuit::Instance,
    keys::SpendValidatingKey,
    note::{Note, TransmittedNoteCiphertext},
    tree::MerklePath,
    value::NoteValue,
    circuit::{Circuit, K}
};
use halo2_proofs::circuit::Value;

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone)]
pub struct Bundle(
    /// The list of raw zactions that make up this bundle.
    Vec<RawZAction>
);

impl Bundle
{
    /// Constructs a `Bundle` from its constituent parts.
    pub fn from_parts(
        actions: Vec<RawZAction>,
    ) -> Self
    {
        assert!(!actions.is_empty());
        Bundle(actions)
    }

    /// Returns the list of actions that make up this bundle.
    pub fn actions(&self) -> &Vec<RawZAction>
    {
        &self.0
    }

    /// Returns the list of public zactions of all actions of this bundle.
    pub fn zactions(&self) -> Vec<ZAction>
    {
        self.0.iter().map(|a| a.zaction()).collect()
    }

    /// Returns the list of encrypted notes of all actions of this bundle
    pub fn encrypted_notes<R: RngCore>(&self, mut rng: R) -> Vec<TransmittedNoteCiphertext>
    {
        let mut v = Vec::new();
        self.0.iter().for_each(|a| v.append(&mut a.encrypted_notes(&mut rng)));
        v
    }

    /// Calculates and returns the proof for this bundle
    pub fn proof<R: RngCore>(&self, mut rng: R) -> (Proof, Vec<Circuit>, Vec<Instance>)
    {
        let mut circuits: Vec<Circuit> = Vec::new();
        let mut instances: Vec<Instance> = Vec::new();

        self.0.iter().for_each(|a| {

            // there is no proof for ZA_MINTAUTH actions
            if a.za_type() != ZA_MINTAUTH
            {
                instances.push(a.zaction().instance());

                let (_dummy_sk, dummy_fvk, dummy_note) = Note::dummy(&mut rng, None, Some(NoteValue::zero()));
                let path = a.auth_path_a().get_or_insert(&MerklePath::dummy(&mut rng)).clone();
                let fvk = if a.za_type() == ZA_MINTFT || a.za_type() == ZA_MINTNFT || a.za_type() == ZA_BURNAUTH { dummy_fvk } else { a.fvk() };
                let note_a = *a.note_a().get_or_insert(dummy_note);
                let note_b = *a.note_b().get_or_insert(dummy_note);
                let note_c = *a.note_c().get_or_insert(dummy_note);
                
                let ak: SpendValidatingKey = fvk.clone().into();
                let nk = *fvk.nk();
                // if this fails the spending authority (derived from fvk) for note a is wrong:
                // note_a's address was not derived from this fvk
                let rivk = fvk.rivk(fvk.scope_for_address(&note_a.recipient()).unwrap());
                

                circuits.push(Circuit {
                    path: Value::known(path.auth_path()),
                    pos: Value::known(path.position()),
                    g_d_a: Value::known(note_a.recipient().g_d()),
                    pk_d_a: Value::known(*note_a.recipient().pk_d()),
                    d1_a: Value::known(note_a.d1()),
                    d2_a: Value::known(note_a.d2()),
                    rho_a: Value::known(note_a.rho()),
                    psi_a: Value::known(note_a.rseed().psi(&note_a.rho())),
                    rcm_a: Value::known(note_a.rseed().rcm(&note_a.rho())),
                    cm_a: Value::known(note_a.commitment()),
                    alpha: Value::known(a.alpha_a()),
                    ak: Value::known(ak),
                    nk: Value::known(nk),
                    rivk: Value::known(rivk),
                    g_d_b: Value::known(note_b.recipient().g_d()),
                    pk_d_b: Value::known(*note_b.recipient().pk_d()),
                    d1_b: Value::known(note_b.d1()),
                    d2_b: Value::known(note_b.d2()),
                    sc_b: Value::known(note_b.sc()),
                    rho_b: Value::known(note_b.rho()),
                    psi_b: Value::known(note_b.rseed().psi(&note_b.rho())),
                    rcm_b: Value::known(note_b.rseed().rcm(&note_b.rho())),
                    g_d_c: Value::known(note_c.recipient().g_d()),
                    pk_d_c: Value::known(*note_c.recipient().pk_d()),
                    d1_c: Value::known(note_c.d1()),
                    psi_c: Value::known(note_c.rseed().psi(&note_c.rho())),
                    rcm_c: Value::known(note_c.rseed().rcm(&note_c.rho())),
                });
            }
        });

        assert!(circuits.len() > 0, "bundle must contain than just ZA_MINTAUTH");
        let pk = ProvingKey::build(Circuit::default(), K);
        (Proof::create(&pk, &circuits, &instances, rng).unwrap(), circuits, instances)
    }

    /// Prepares a bundle for private transaction by calculating proof, the list of zactions and the encrypted note data
    pub fn prepare<R: RngCore>(&self, mut rng: R) -> ((Proof, Vec<Circuit>, Vec<Instance>), Vec<ZAction>, Vec<TransmittedNoteCiphertext>)
    {
        (self.proof(&mut rng), self.zactions(), self.encrypted_notes(&mut rng))
    }
}

#[cfg(test)]
mod tests
{
    use rand::rngs::OsRng;
    use zeos_verifier::verify_zeos_proof;
    use super::{Bundle, RawZAction};
    use crate::{
        keys::{
            SpendingKey, FullViewingKey, Scope::External
        },
        note_encryption::{
            try_note_decryption, try_output_recovery_with_ovk
        },
        note::{NT_FT, NT_NFT, NT_AT, Note, Nullifier},
        value::{NoteValue}, 
        action::{ZA_MINTFT, ZA_MINTNFT, ZA_MINTAUTH, ZA_TRANSFERFT, ZA_TRANSFERNFT, ZA_BURNFT, ZA_BURNFT2, ZA_BURNNFT, ZA_BURNAUTH},
        tree::MerklePath,
        circuit::{Circuit, K}
    };
    use rustzeos::halo2::{VerifyingKey, Instance};
    use halo2_proofs::dev::MockProver;
    
    #[test]
    fn ft_transfer()
    {
        let mut rng = OsRng.clone();

        // Alice' key material
        let sk_alice = SpendingKey::from_zip32_seed("This is Alice seed string! Usually this is just a listing of words. Here we just use sentences.".as_bytes(), 0, 0).unwrap();
        let fvk_alice = FullViewingKey::from(&sk_alice);
        let alice = fvk_alice.address_at(0u32, External);

        // Bob's key material
        let sk_bob = SpendingKey::from_zip32_seed("This is Bob's seed string. His seed is a little shorter...".as_bytes(), 0, 0).unwrap();
        let fvk_bob = FullViewingKey::from(&sk_bob);
        let bob = fvk_bob.address_at(0u32, External);

        // Alice Note material
        let note1 = Note::new(
            NT_FT, 
            alice,
            NoteValue::from_raw(5),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );
        let note2 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(3),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );
        let note3 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(2),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );

        // Alice has 5 + 3 + 2 = 10 and sends 9 (5+3+1) to Bob
        // note1 = note4 + note5 = 5 + 0
        let note4 = Note::new(
            NT_FT,
            bob,
            NoteValue::from_raw(5),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [4; 512]
        );
        let note5 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(0),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [5; 512]
        );
        // note2 = note6 + note7 = 3 + 0
        let note6 = Note::new(
            NT_FT,
            bob,
            NoteValue::from_raw(3),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note2.nullifier(&fvk_alice),
            rng,
            [6; 512]
        );
        let note7 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(0),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note2.nullifier(&fvk_alice),
            rng,
            [7; 512]
        );
        // note3 = note8 + note9 = 1 + 1
        let note8 = Note::new(
            NT_FT,
            bob,
            NoteValue::from_raw(1),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note3.nullifier(&fvk_alice),
            rng,
            [8; 512]
        );
        let note9 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(1),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            note3.nullifier(&fvk_alice),
            rng,
            [9; 512]
        );
        
        // create the 3 private fungible token transfer actions
        let path = MerklePath::dummy(&mut rng);
        let action1 = RawZAction::from_parts(ZA_TRANSFERFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note1), 
            Some(note4), 
            Some(note5), 
            "".to_string(), 
            rng);
        let action2 = RawZAction::from_parts(ZA_TRANSFERFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note2), 
            Some(note6), 
            Some(note7), 
            "".to_string(), 
            rng);
        let action3 = RawZAction::from_parts(ZA_TRANSFERFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note3), 
            Some(note8), 
            Some(note9), 
            "".to_string(), 
            rng);
        
        // create Bundle and prepare for transaction
        let mut v = Vec::new();
        v.push(action1);
        v.push(action2);
        v.push(action3);
        let bundle = Bundle::from_parts(v);
        let ((proof, circuits, instances), zactions, encrypted_notes) = bundle.prepare(rng);

        // check bundle parts
        assert_eq!(zactions.len(), 3);
        assert_eq!(encrypted_notes.len(), 6);

        // mock prover
        for (circuit, instance) in circuits.iter().zip(instances.iter())
        {
            assert_eq!(
                MockProver::run(
                    K,
                    circuit,
                    instance
                        .to_halo2_instance()
                        .iter()
                        .map(|p| p.to_vec())
                        .collect()
                )
                .unwrap()
                .verify(),
                Ok(())
            );
        }
        // verify proof
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance_vec()).collect();
        let vk = VerifyingKey::build(Circuit::default(), K);
        assert!(proof.verify(&vk, &instances).is_ok());

        // test receiver decryption
        match try_note_decryption(&fvk_bob.to_ivk(External), &encrypted_notes[0]) {
            Some(decrypted_note) => { assert_eq!(decrypted_note, note4); assert_eq!(&decrypted_note.memo(), &[4; 512]);},
            None => panic!("Note4 decryption failed"),
        }
        match try_note_decryption(&fvk_alice.to_ivk(External), &encrypted_notes[1]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note5),
            None => panic!("Note5 decryption failed"),
        }
        match try_note_decryption(&fvk_bob.to_ivk(External), &encrypted_notes[2]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note6),
            None => panic!("Note6 decryption failed"),
        }
        match try_note_decryption(&fvk_alice.to_ivk(External), &encrypted_notes[3]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note7),
            None => panic!("Note7 decryption failed"),
        }
        match try_note_decryption(&fvk_bob.to_ivk(External), &encrypted_notes[4]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note8),
            None => panic!("Note8 decryption failed"),
        }
        match try_note_decryption(&fvk_alice.to_ivk(External), &encrypted_notes[5]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note9),
            None => panic!("Note9 decryption failed"),
        }

        // test sender decryption
        match try_output_recovery_with_ovk(&fvk_alice.to_ovk(External), &encrypted_notes[0]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note4),
            None => panic!("Output4 recovery failed"),
        }
        match try_output_recovery_with_ovk(&fvk_alice.to_ovk(External), &encrypted_notes[2]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note6),
            None => panic!("Output6 recovery failed"),
        }
        match try_output_recovery_with_ovk(&fvk_alice.to_ovk(External), &encrypted_notes[4]) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note8),
            None => panic!("Output8 recovery failed"),
        }
    }

    #[test]
    fn all_types()
    {
        let mut rng = OsRng.clone();

        // Alice' key material
        let sk_alice = SpendingKey::from_zip32_seed("This is Alice seed string! Usually this is just a listing of words. Here we just use sentences.".as_bytes(), 0, 0).unwrap();
        let fvk_alice = FullViewingKey::from(&sk_alice);
        let alice = fvk_alice.address_at(0u32, External);

        // Bob's key material
        let sk_bob = SpendingKey::from_zip32_seed("This is Bob's seed string. His seed is a little shorter...".as_bytes(), 0, 0).unwrap();
        let fvk_bob = FullViewingKey::from(&sk_bob);
        let bob = fvk_bob.address_at(0u32, External);

        // dummy recepient
        let dummy = Note::dummy(&mut rng, None, None).2.recipient();

        // Alice Note material
        // mint ft
        let note1 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(10000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );
        // mint nft
        let note2 = Note::new(
            NT_NFT,
            alice,
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(1),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );
        // mint auth
        let note3 = Note::new(
            NT_AT,
            alice,
            NoteValue::from_raw(0),
            NoteValue::from_raw(0),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(1),
            Nullifier::dummy(&mut rng),
            rng,
            [0; 512]
        );
        // transfer ft (note1: 10000 = 2000 + 8000)
        let note4 = Note::new(
            NT_FT,
            bob,
            NoteValue::from_raw(2000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        let note5 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(8000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        // transfer nft (note2)
        let note6 = Note::new(
            NT_NFT,
            bob,
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(1),
            note2.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        // burn ft (note1: 10000 = 2000 + 8000)
        let note7 = Note::new(
            NT_FT,
            dummy,
            NoteValue::from_raw(2000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        let note8 = Note::new(
            NT_FT,
            alice,
            NoteValue::from_raw(8000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        // burn ft 2 (note1: 10000 = 2000 + 8000)
        let note9 = Note::new(
            NT_FT,
            dummy,
            NoteValue::from_raw(2000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        let note10 = Note::new(
            NT_FT,
            dummy,
            NoteValue::from_raw(8000),
            NoteValue::from_raw(357812207620),
            NoteValue::from_raw(6138663591592764928),
            NoteValue::from_raw(0),
            note1.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        // burn nft (note2)
        let note11 = Note::new(
            NT_NFT,
            dummy,
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(1),
            note2.nullifier(&fvk_alice),
            rng,
            [0; 512]
        );
        
        // create all the ZEOS actions using a dummy path
        let path = MerklePath::dummy(&mut rng);
        let action1 = RawZAction::from_parts(ZA_MINTFT, 
            &fvk_alice,
            None, 
            None, 
            Some(note1), 
            None, 
            "".to_string(), 
            rng);
        let action2 = RawZAction::from_parts(ZA_MINTNFT, 
            &fvk_alice,  
            None, 
            None, 
            Some(note2), 
            None, 
            "".to_string(), 
            rng);
        let action3 = RawZAction::from_parts(ZA_MINTAUTH, 
            &fvk_alice,
            None, 
            None, 
            Some(note3), 
            None, 
            "".to_string(), 
            rng);
        let action4 = RawZAction::from_parts(ZA_TRANSFERFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note1), 
            Some(note4), 
            Some(note5), 
            "".to_string(), 
            rng);
        let action5 = RawZAction::from_parts(ZA_TRANSFERNFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note2), 
            Some(note6), 
            None, 
            "".to_string(), 
            rng);
        let action6 = RawZAction::from_parts(ZA_BURNFT, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note1), 
            Some(note7), 
            Some(note8), 
            "".to_string(), 
            rng);
        let action7 = RawZAction::from_parts(ZA_BURNFT2, 
            &fvk_alice,
            Some(path.clone()), 
            Some(note1), 
            Some(note9), 
            Some(note10), 
            "".to_string(), 
            rng);
        let action8 = RawZAction::from_parts(ZA_BURNNFT,
            &fvk_alice,
            Some(path.clone()), 
            Some(note2), 
            Some(note11), 
            None, 
            "".to_string(), 
            rng);
        let action9 = RawZAction::from_parts(ZA_BURNAUTH,
            &fvk_alice,
            None, 
            None, 
            Some(note3), 
            None, 
            "".to_string(), 
            rng);
        
        // create Bundle and prepare for transaction
        let mut v = Vec::new();
        v.push(action1);
        v.push(action2);
        v.push(action3);
        v.push(action4);
        v.push(action5);
        v.push(action6);
        v.push(action7);
        v.push(action8);
        v.push(action9);
        let bundle = Bundle::from_parts(v);
        let ((proof, circuits, instances), zactions, encrypted_notes) = bundle.prepare(rng);

        // check bundle parts
        assert_eq!(zactions.len(), 9);
        assert_eq!(encrypted_notes.len(), 7);

        // mock prover
        for (circuit, instance) in circuits.iter().zip(instances.iter())
        {
            assert_eq!(
                MockProver::run(
                    K,
                    circuit,
                    instance
                        .to_halo2_instance()
                        .iter()
                        .map(|p| p.to_vec())
                        .collect()
                )
                .unwrap()
                .verify(),
                Ok(())
            );
        }
        // verify proof
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance_vec()).collect();
        let vk = VerifyingKey::build(Circuit::default(), K);
        assert!(proof.verify(&vk, &instances).is_ok());

        // verify proof using zeos verifier
        const ZI_SIZE: usize = 32 + 32 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 32 + 32;
        let mut inputs_str = "".to_string();
        for za in zactions
        {
            if za.za_type() != ZA_MINTAUTH
            {
                let za_str: String = za.serialize_eos().chars().skip(16).take(ZI_SIZE*2).collect();
                inputs_str.push_str(&za_str);
            }
        }
        let mut inputs = vec![0; inputs_str.len()/2];
        assert!(hex::decode_to_slice(inputs_str, &mut inputs).is_ok());
        let mut arr = Vec::new();
        vk.serialize(&mut arr);
        assert!(verify_zeos_proof(proof.as_ref(), &inputs, &arr));
        
    }
}
