//! The ZEOS Action circuit implementation.

use group::{Curve, GroupEncoding};
use halo2_proofs::{
    circuit::{floor_planner, Layouter, Value},
    plonk::{
        self, Advice, Column, Constraints, Instance as InstanceColumn,
        Selector
    },
    poly::Rotation,
};
use pasta_curves::{arithmetic::CurveAffine, pallas, vesta};

use self::{
    commit_ivk::{CommitIvkChip, CommitIvkConfig},
    gadget::{
        add_chip::{AddChip, AddConfig},
        assign_free_advice, assign_advice_from_instance,
    },
    note_commit::{NoteCommitChip, NoteCommitConfig},
};
use crate::{
    constants::{
        OrchardCommitDomains, OrchardFixedBases, OrchardFixedBasesFull, OrchardHashDomains,
        MERKLE_DEPTH_ORCHARD,
    },
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        ExtractedNoteCommitment,
    },
    primitives::redpallas::{SpendAuth, VerificationKey},
    spec::NonIdentityPallasPoint,
    tree::{Anchor, MerkleHashOrchard},
    value::{NoteValue},
};
use halo2_gadgets::{
    ecc::{
        chip::{EccChip, EccConfig},
        FixedPoint, NonIdentityPoint, Point, ScalarFixed, ScalarVar,
    },
    poseidon::{primitives as poseidon, Pow5Chip as PoseidonChip, Pow5Config as PoseidonConfig},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::{
            chip::{MerkleChip, MerkleConfig},
            MerklePath,
        },
    },
    utilities::lookup_range_check::LookupRangeCheckConfig,
};

mod commit_ivk;
pub mod gadget;
mod note_commit;

/// Size of the ZEOS circuit.
pub const K: u32 = 11;

// Absolute offsets for public inputs.
const ANCHOR: usize = 0;
const NF: usize = 1;
const RK_X: usize = 2;
const RK_Y: usize = 3;
const NFT: usize = 4;
const B_D1: usize = 5;
const B_D2: usize = 6;
const B_SC: usize = 7;
const C_D1: usize = 8;
const CMB: usize = 9;
const CMC: usize = 10;

/// Configuration needed to use the ZEOS Action circuit.
#[derive(Clone, Debug)]
pub struct Config {
    primary: Column<InstanceColumn>,
    q_orchard: Selector,
    advices: [Column<Advice>; 10],
    add_config: AddConfig,
    ecc_config: EccConfig<OrchardFixedBases>,
    poseidon_config: PoseidonConfig<pallas::Base, 3, 2>,
    merkle_config_1: MerkleConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
    merkle_config_2: MerkleConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
    sinsemilla_config_1: SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
    sinsemilla_config_2: SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
    commit_ivk_config: CommitIvkConfig,
    a_note_commit_config: NoteCommitConfig,
    b_note_commit_config: NoteCommitConfig,
    c_note_commit_config: NoteCommitConfig,
}

/// The ZEOS Action circuit.
#[derive(Clone, Debug, Default)]
pub struct Circuit {
    // A
    pub path: Value<[MerkleHashOrchard; MERKLE_DEPTH_ORCHARD]>,
    pub pos: Value<u32>,
    pub g_d_a: Value<NonIdentityPallasPoint>,
    pub pk_d_a: Value<DiversifiedTransmissionKey>,
    pub d1_a: Value<NoteValue>,
    pub d2_a: Value<NoteValue>,
    pub rho_a: Value<Nullifier>,
    pub psi_a: Value<pallas::Base>,
    pub rcm_a: Value<NoteCommitTrapdoor>,
    pub cm_a: Value<NoteCommitment>,
    pub alpha: Value<pallas::Scalar>,
    pub ak: Value<SpendValidatingKey>,
    pub nk: Value<NullifierDerivingKey>,
    pub rivk: Value<CommitIvkRandomness>,
    // B
    pub g_d_b: Value<NonIdentityPallasPoint>,
    pub pk_d_b: Value<DiversifiedTransmissionKey>,
    pub d1_b: Value<NoteValue>,
    pub d2_b: Value<NoteValue>,
    pub sc_b: Value<NoteValue>,
    pub rho_b: Value<Nullifier>,
    pub psi_b: Value<pallas::Base>,
    pub rcm_b: Value<NoteCommitTrapdoor>,
    // C
    pub g_d_c: Value<NonIdentityPallasPoint>,
    pub pk_d_c: Value<DiversifiedTransmissionKey>,
    pub d1_c: Value<NoteValue>,
    pub psi_c: Value<pallas::Base>,
    pub rcm_c: Value<NoteCommitTrapdoor>,
}

impl plonk::Circuit<pallas::Base> for Circuit {
    type Config = Config;
    type FloorPlanner = floor_planner::V1;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        // Advice columns used in the Orchard circuit.
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // Constrain all top level signals
        let q_orchard = meta.selector();
        meta.create_gate("ZEOS circuit checks", |meta| {
            let q_orchard       = meta.query_selector(q_orchard);
            let d1_a            = meta.query_advice(advices[0], Rotation::cur());
            let d1_b            = meta.query_advice(advices[1], Rotation::cur());
            let d1_c            = meta.query_advice(advices[2], Rotation::cur());
            let root            = meta.query_advice(advices[3], Rotation::cur());
            let anchor          = meta.query_advice(advices[4], Rotation::cur());
            let cm_a            = meta.query_advice(advices[5], Rotation::cur());
            let derived_cm_a    = meta.query_advice(advices[6], Rotation::cur());
            let pk_d_a          = meta.query_advice(advices[7], Rotation::cur());
            let derived_pk_d_a  = meta.query_advice(advices[8], Rotation::cur());
            let rk_x            = meta.query_advice(advices[9], Rotation::cur());
            let rk_x_i          = meta.query_advice(advices[0], Rotation::next());
            let rk_y            = meta.query_advice(advices[1], Rotation::next());
            let rk_y_i          = meta.query_advice(advices[2], Rotation::next());
            let d2_a            = meta.query_advice(advices[3], Rotation::next());
            let d2_b            = meta.query_advice(advices[4], Rotation::next());
            let nf_a            = meta.query_advice(advices[5], Rotation::next());
            let rho_b           = meta.query_advice(advices[6], Rotation::next());
            let nf              = meta.query_advice(advices[7], Rotation::next());
            let b_d1            = meta.query_advice(advices[8], Rotation::next());
            let b_d2            = meta.query_advice(advices[9], Rotation::next());
            let b_sc            = meta.query_advice(advices[0], Rotation(2));
            let sc_b            = meta.query_advice(advices[1], Rotation(2));
            let cmb             = meta.query_advice(advices[2], Rotation(2));
            let cm_b            = meta.query_advice(advices[3], Rotation(2));
            let nft             = meta.query_advice(advices[4], Rotation(2));
            let cmc             = meta.query_advice(advices[5], Rotation(2));
            let cm_c            = meta.query_advice(advices[6], Rotation(2));
            let c_d1            = meta.query_advice(advices[7], Rotation(2));

            //let one             = Expression::Constant(pallas::Base::one());
            //let zero            = Expression::Constant(pallas::Base::zero());

            Constraints::with_selector(
                q_orchard,
                [
                    (
                        "Either a = b + c, or a = c = 0",
                        (d1_a.clone() - d1_b.clone() - d1_c.clone()) * (d1_a.clone() + d1_c.clone()),
                    ),
                    (
                        "Either d1_a = 0, or root = anchor",
                        d1_a.clone() * (root - anchor),
                    ),
                    (
                        "Either d1_a = 0, or cm_a = derived_cm_a",
                        d1_a.clone() * (cm_a - derived_cm_a),
                    ),
                    (
                        "Either d1_a = 0, or pk_d_a = derived_pk_d_a",
                        d1_a.clone() * (pk_d_a - derived_pk_d_a),
                    ),
                    (
                        "Either d1_a = 0, or rk_x = rk_x_i",
                        d1_a.clone() * (rk_x - rk_x_i),
                    ),
                    (
                        "Either d1_a = 0, or rk_y = rk_y_i",
                        d1_a.clone() * (rk_y - rk_y_i),
                    ),
                    (
                        "Either d1_a = 0, or d2_a = d2_b",
                        d1_a.clone() * (d2_a - d2_b.clone()),
                    ),
                    (
                        "Either d1_a = 0, or nf_a = rho_b = nf",
                        d1_a.clone() * (nf_a + rho_b - nf.clone() - nf),
                    ),
                    (
                        "Either b_d1 = 0, or b_d1 = d1_b",
                        b_d1.clone() * (b_d1.clone() - d1_b),
                    ),
                    (
                        "Either b_d1 = 0, or b_d2 = d2_b",
                        b_d1.clone() * (b_d2 - d2_b),
                    ),
                    (
                        "Either b_d1 = 0, or b_sc = sc_b",
                        b_d1.clone() * (b_sc - sc_b),
                    ),
                    (
                        "Either cmb = 0, or cmb = cm_b",
                        cmb.clone() * (cmb - cm_b),
                    ),
                    (
                        "nft = 0 or c = 0",
                        nft * d1_c.clone(),
                    ),
                    (
                        "Either c_d1 = 0, or c_d1 = d1_c",
                        c_d1.clone() * (c_d1 - d1_c),
                    ),
                    (
                        "Either cmc = 0, or cmc = cm_c",
                        cmc.clone() * (cmc - cm_c),
                    ),
                ],
            )
        });

        // Addition of two field elements.
        let add_config = AddChip::configure(meta, advices[7], advices[8], advices[6]);

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.lookup_table_column();
        let lookup = (
            table_idx,
            meta.lookup_table_column(),
            meta.lookup_table_column(),
        );

        // Instance column used for public inputs
        let primary = meta.instance_column();
        meta.enable_equality(primary);

        // Permutation over all advice columns.
        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }

        // Poseidon requires four advice columns, while ECC incomplete addition requires
        // six, so we could choose to configure them in parallel. However, we only use a
        // single Poseidon invocation, and we have the rows to accommodate it serially.
        // Instead, we reduce the proof size by sharing fixed columns between the ECC and
        // Poseidon chips.
        let lagrange_coeffs = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let rc_a = lagrange_coeffs[2..5].try_into().unwrap();
        let rc_b = lagrange_coeffs[5..8].try_into().unwrap();

        // Also use the first Lagrange coefficient column for loading global constants.
        // It's free real estate :)
        meta.enable_constant(lagrange_coeffs[0]);

        // We have a lot of free space in the right-most advice columns; use one of them
        // for all of our range checks.
        let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config = EccChip::<OrchardFixedBases>::configure(meta, advices, lagrange_coeffs, range_check);

        // Configuration for the Poseidon hash.
        let poseidon_config = PoseidonChip::configure::<poseidon::P128Pow5T3>(
            meta,
            // We place the state columns after the partial_sbox column so that the
            // pad-and-add region can be laid out more efficiently.
            advices[6..9].try_into().unwrap(),
            advices[5],
            rc_a,
            rc_b,
        );

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_1, merkle_config_1) = {
            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[6],
                lagrange_coeffs[0],
                lookup,
                range_check,
            );
            let merkle_config_1 = MerkleChip::configure(meta, sinsemilla_config_1.clone());

            (sinsemilla_config_1, merkle_config_1)
        };

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_2, merkle_config_2) = {
            let sinsemilla_config_2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                lagrange_coeffs[1],
                lookup,
                range_check,
            );
            let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

            (sinsemilla_config_2, merkle_config_2)
        };

        // Configuration to handle decomposition and canonicity checking
        // for CommitIvk.
        let commit_ivk_config = CommitIvkChip::configure(meta, advices);

        // Configuration to handle decomposition and canonicity checking for NoteCommit_a.
        let a_note_commit_config = NoteCommitChip::configure(meta, advices, sinsemilla_config_1.clone());

        // Configuration to handle decomposition and canonicity checking for NoteCommit_b.
        let b_note_commit_config = NoteCommitChip::configure(meta, advices, sinsemilla_config_2.clone());

        // Configuration to handle decomposition and canonicity checking for NoteCommit_c.
        let c_note_commit_config = NoteCommitChip::configure(meta, advices, sinsemilla_config_1.clone());

        Config {
            primary,
            q_orchard,
            advices,
            add_config,
            ecc_config,
            poseidon_config,
            merkle_config_1,
            merkle_config_2,
            sinsemilla_config_1,
            sinsemilla_config_2,
            commit_ivk_config,
            a_note_commit_config,
            b_note_commit_config,
            c_note_commit_config,
        }
    }

    #[allow(non_snake_case)]
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), &mut layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip();

        // Witness nft from instance column
        let nft = assign_advice_from_instance(
            layouter.namespace(|| "witness pub nft"),
            config.primary,
            NFT,
            config.advices[0]
        )?;

        // Witness psi_a
        let psi_a = assign_free_advice(
            layouter.namespace(|| "witness psi_a"),
            config.advices[0],
            self.psi_a,
        )?;

        // Witness rho_a
        let rho_a = assign_free_advice(
            layouter.namespace(|| "witness rho_a"),
            config.advices[0],
            self.rho_a.map(|rho| rho.0),
        )?;

        // Witness cm_a
        let cm_a = Point::new(
            ecc_chip.clone(),
            layouter.namespace(|| "witness cm_a"),
            self.cm_a.as_ref().map(|cm| cm.inner().to_affine()),
        )?;

        // Witness g_d_a
        let g_d_a = NonIdentityPoint::new(
            ecc_chip.clone(),
            layouter.namespace(|| "witness g_d_a"),
            self.g_d_a.as_ref().map(|gd| gd.to_affine()),
        )?;

        // Witness pk_d_a
        let pk_d_a = NonIdentityPoint::new(
            ecc_chip.clone(),
            layouter.namespace(|| "witness pk_d_a"),
            self.pk_d_a.map(|pk_d| pk_d.inner().to_affine()),
        )?;

        // Witness ak_P
        let ak_P: Value<pallas::Point> = self.ak.as_ref().map(|ak| ak.into());
        let ak_P = NonIdentityPoint::new(
            ecc_chip.clone(),
            layouter.namespace(|| "witness ak_P"),
            ak_P.map(|ak_P| ak_P.to_affine()),
        )?;

        // Witness nk
        let nk = assign_free_advice(
            layouter.namespace(|| "witness nk"),
            config.advices[0],
            self.nk.map(|nk| nk.inner()),
        )?;

        // Witness d1_a
        let d1_a = assign_free_advice(
            layouter.namespace(|| "witness d1_a"),
            config.advices[0],
            self.d1_a,
        )?;

        // Witness d2_a
        let d2_a = assign_free_advice(
            layouter.namespace(|| "witness d2_a"),
            config.advices[0],
            self.d2_a,
        )?;

        // Witness d1_b
        let d1_b = assign_free_advice(
            layouter.namespace(|| "witness d1_b"),
            config.advices[0],
            self.d1_b,
        )?;

        // Witness d2_b
        let d2_b = assign_free_advice(
            layouter.namespace(|| "witness d2_b"),
            config.advices[0],
            self.d2_b,
        )?;

        // Witness sc_b
        let sc_b = assign_free_advice(
            layouter.namespace(|| "witness sc_b"),
            config.advices[0],
            self.sc_b,
        )?;

        // Witness rho_b
        let rho_b = assign_free_advice(
            layouter.namespace(|| "witness rho_b"),
            config.advices[0],
            self.rho_b.map(|rho| rho.0),
        )?;

        // Witness d1_c
        let d1_c = assign_free_advice(
            layouter.namespace(|| "witness d1_c"),
            config.advices[0],
            self.d1_c,
        )?;

        // Merkle path validity check (https://p.z.cash/ZKS:action-merkle-path-validity?partial).
        let root = {
            let path = self
                .path
                .map(|typed_path| typed_path.map(|node| node.inner()));
            let merkle_inputs = MerklePath::construct(
                [config.merkle_chip_1(), config.merkle_chip_2()],
                OrchardHashDomains::MerkleCrh,
                self.pos,
                path,
            );
            let leaf = cm_a.extract_p().inner().clone();
            merkle_inputs.calculate_root(layouter.namespace(|| "Merkle path"), leaf)?
        };

        // Nullifier integrity (https://p.z.cash/ZKS:action-nullifier-integrity).
        let nf_a = gadget::derive_nullifier(
            layouter.namespace(|| "nf_a = DeriveNullifier_nk(rho_a, psi_a, cm_a)"),
            config.poseidon_chip(),
            config.add_chip(),
            ecc_chip.clone(),
            rho_a.clone(),
            &psi_a,
            &cm_a,
            nk.clone(),
        )?;

        // Spend authority (https://p.z.cash/ZKS:action-spend-authority)
        let rk = {
            let alpha =
                ScalarFixed::new(ecc_chip.clone(), layouter.namespace(|| "alpha"), self.alpha)?;

            // alpha_commitment = [alpha] SpendAuthG
            let (alpha_commitment, _) = {
                let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
                let spend_auth_g = FixedPoint::from_inner(ecc_chip.clone(), spend_auth_g);
                spend_auth_g.mul(layouter.namespace(|| "[alpha] SpendAuthG"), alpha)?
            };

            // [alpha] SpendAuthG + ak_P
            let rk = alpha_commitment.add(layouter.namespace(|| "rk"), &ak_P)?;

            rk
        };

        // Diversified address integrity (https://p.z.cash/ZKS:action-addr-integrity?partial).
        let derived_pk_d_a = {
            let ivk = {
                let ak = ak_P.extract_p().inner().clone();
                let rivk = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rivk"),
                    self.rivk.map(|rivk| rivk.inner()),
                )?;

                gadget::commit_ivk(
                    config.sinsemilla_chip_1(),
                    ecc_chip.clone(),
                    config.commit_ivk_chip(),
                    layouter.namespace(|| "CommitIvk"),
                    ak,
                    nk,
                    rivk,
                )?
            };
            let ivk =
                ScalarVar::from_base(ecc_chip.clone(), layouter.namespace(|| "ivk"), ivk.inner())?;

            // [ivk] g_d_a
            // The scalar value is passed through and discarded.
            let (derived_pk_d_a, _ivk) =
                g_d_a.mul(layouter.namespace(|| "[ivk] g_d_a"), ivk)?;

            // Constrain derived pk_d_a to equal witnessed pk_d_a
            //
            // This equality constraint is technically superfluous, because the assigned
            // value of `derived_pk_d_a` is an equivalent witness. But it's nice to see
            // an explicit connection between circuit-synthesized values, and explicit
            // prover witnesses. We could get the best of both worlds with a write-on-copy
            // abstraction (https://github.com/zcash/halo2/issues/334).
            //let pk_d_a = NonIdentityPoint::new(
            //    ecc_chip.clone(),
            //    layouter.namespace(|| "witness pk_d_a"),
            //    self.pk_d_a.map(|pk_d| pk_d.inner().to_affine()),
            //)?;
            //derived_pk_d_a.constrain_equal(layouter.namespace(|| "pk_d_a equality"), &pk_d_a)?;

            derived_pk_d_a
        };

        // note A commitment integrity (https://p.z.cash/ZKS:action-cm-old-integrity?partial).
        let derived_cm_a = {
            let rcm_a = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "rcm_a"),
                self.rcm_a.as_ref().map(|rcm| rcm.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let derived_cm_a = gadget::note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_1(),
                config.ecc_chip(),
                config.note_commit_chip_a(),
                g_d_a.inner(),
                pk_d_a.inner(),
                d1_a.clone(),
                rho_a.clone(),
                psi_a,
                d2_a.clone(),
                nft.clone(),
                sc_b.clone(),
                rcm_a,
            )?;

            derived_cm_a
        };

        // note B commitment integrity (https://p.z.cash/ZKS:action-cmx-new-integrity?partial).

        // Witness g_d_b
        let g_d_b = {
            let g_d_b = self.g_d_b.map(|g_d_b| g_d_b.to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness g_d_b_star"),
                g_d_b,
            )?
        };

        // Witness pk_d_b
        let pk_d_b = {
            let pk_d_b = self.pk_d_b.map(|pk_d_b| pk_d_b.inner().to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_b"),
                pk_d_b,
            )?
        };

        // Witness psi_b
        let psi_b = assign_free_advice(
            layouter.namespace(|| "witness psi_b"),
            config.advices[0],
            self.psi_b,
        )?;

        // Witness rcm_b
        let rcm_b = ScalarFixed::new(
            ecc_chip.clone(),
            layouter.namespace(|| "rcm_b"),
            self.rcm_b.as_ref().map(|rcm| rcm.inner()),
        )?;

        // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
        let cm_b = gadget::note_commit(
            layouter.namespace(|| {
                "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
            }),
            config.sinsemilla_chip_2(),
            config.ecc_chip(),
            config.note_commit_chip_b(),
            g_d_b.inner(),
            pk_d_b.inner(),
            d1_b.clone(),
            rho_b.clone(),
            psi_b,
            d2_b.clone(),
            nft.clone(),
            sc_b.clone(),
            rcm_b,
        )?;
        
        // note C commitment integrity (https://p.z.cash/ZKS:action-cmx-new-integrity?partial).

        // Witness g_d_c
        let g_d_c = {
            let g_d_c = self.g_d_c.map(|g_d_c| g_d_c.to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness g_d_c_star"),
                g_d_c,
            )?
        };

        // Witness pk_d_c
        let pk_d_c = {
            let pk_d_c = self.pk_d_c.map(|pk_d_c| pk_d_c.inner().to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_c"),
                pk_d_c,
            )?
        };

        // Witness psi_c
        let psi_c = assign_free_advice(
            layouter.namespace(|| "witness psi_c"),
            config.advices[0],
            self.psi_c,
        )?;

        // Witness rcm_c
        let rcm_c = ScalarFixed::new(
            ecc_chip,
            layouter.namespace(|| "rcm_c"),
            self.rcm_c.as_ref().map(|rcm| rcm.inner()),
        )?;

        // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
        let cm_c = gadget::note_commit(
            layouter.namespace(|| {
                "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
            }),
            config.sinsemilla_chip_1(),
            config.ecc_chip(),
            config.note_commit_chip_c(),
            g_d_c.inner(),
            pk_d_c.inner(),
            d1_c.clone(),
            nf_a.inner().clone(),
            psi_c,
            d2_a.clone(),
            nft.clone(),
            sc_b.clone(),
            rcm_c,
        )?;

        // Constrain the remaining ZEOS circuit checks.
        layouter.assign_region(
            || "ZEOS circuit checks",
            |mut region| {
                d1_a.copy_advice(|| "d1_a", &mut region, config.advices[0], 0)?;
                d1_b.copy_advice(|| "d1_b", &mut region, config.advices[1], 0)?;
                d1_c.copy_advice(|| "d1_c", &mut region, config.advices[2], 0)?;

                root.copy_advice(|| "calculated root", &mut region, config.advices[3], 0)?;
                region.assign_advice_from_instance(
                    || "pub input anchor",
                    config.primary,
                    ANCHOR,
                    config.advices[4],
                    0,
                )?;

                cm_a.extract_p().inner().copy_advice(|| "cm_a", &mut region, config.advices[5], 0)?;
                derived_cm_a.extract_p().inner().copy_advice(|| "derived_cm_a", &mut region, config.advices[6], 0)?;

                pk_d_a.extract_p().inner().copy_advice(|| "pk_d_a", &mut region, config.advices[7], 0)?;
                derived_pk_d_a.extract_p().inner().copy_advice(|| "derived_pk_d_a", &mut region, config.advices[8], 0)?;

                rk.inner().x().copy_advice(|| "rk_x", &mut region, config.advices[9], 0)?;
                region.assign_advice_from_instance(
                    || "pub input rk_x",
                    config.primary,
                    RK_X,
                    config.advices[0],
                    1,
                )?;

                rk.inner().y().copy_advice(|| "rk_y", &mut region, config.advices[1], 1)?;
                region.assign_advice_from_instance(
                    || "pub input rk_y",
                    config.primary,
                    RK_Y,
                    config.advices[2],
                    1,
                )?;

                d2_a.copy_advice(|| "d2_a", &mut region, config.advices[3], 1)?;
                d2_b.copy_advice(|| "d2_b", &mut region, config.advices[4], 1)?;

                nf_a.inner().copy_advice(|| "nf_a", &mut region, config.advices[5], 1)?;
                rho_b.copy_advice(|| "rho_b", &mut region, config.advices[6], 1)?;
                region.assign_advice_from_instance(
                    || "pub input nf",
                    config.primary,
                    NF,
                    config.advices[7],
                    1,
                )?;

                region.assign_advice_from_instance(
                    || "pub input b_d1",
                    config.primary,
                    B_D1,
                    config.advices[8],
                    1,
                )?;
                region.assign_advice_from_instance(
                    || "pub input b_d2",
                    config.primary,
                    B_D2,
                    config.advices[9],
                    1,
                )?;
                region.assign_advice_from_instance(
                    || "pub input b_sc",
                    config.primary,
                    B_SC,
                    config.advices[0],
                    2,
                )?;
                sc_b.copy_advice(|| "sc_b", &mut region, config.advices[1], 2)?;

                region.assign_advice_from_instance(
                    || "pub input cmb",
                    config.primary,
                    CMB,
                    config.advices[2],
                    2,
                )?;
                cm_b.extract_p().inner().copy_advice(|| "cm_b", &mut region, config.advices[3], 2)?;

                region.assign_advice_from_instance(
                    || "pub input nft",
                    config.primary,
                    NFT,
                    config.advices[4],
                    2,
                )?;

                region.assign_advice_from_instance(
                    || "pub input cmc",
                    config.primary,
                    CMC,
                    config.advices[5],
                    2,
                )?;
                cm_c.extract_p().inner().copy_advice(|| "cm_c", &mut region, config.advices[6], 2)?;

                region.assign_advice_from_instance(
                    || "pub input c_d1",
                    config.primary,
                    C_D1,
                    config.advices[7],
                    2,
                )?;

                config.q_orchard.enable(&mut region, 0)
            },
        )?;

        Ok(())
    }
}

/// Public inputs to the ZEOS Action circuit.
#[derive(Clone, Debug)]
pub struct Instance {
    pub anchor: Anchor,
    pub nf: Nullifier,
    pub rk: VerificationKey<SpendAuth>,
    pub nft: bool,
    pub b_d1: NoteValue,
    pub b_d2: NoteValue,
    pub b_sc: NoteValue,
    pub c_d1: NoteValue,
    pub cmb: ExtractedNoteCommitment,
    pub cmc: ExtractedNoteCommitment,
}

impl Instance {
    /// Constructs an [`Instance`] from its constituent parts.
    ///
    /// This API can be used in combination with [`Proof::verify`] to build verification
    /// pipelines for many proofs, where you don't want to pass around the full bundle.
    /// Use [`Bundle::verify_proof`] instead if you have the full bundle.
    ///
    /// [`Bundle::verify_proof`]: crate::Bundle::verify_proof
    pub fn from_parts(
        anchor: Anchor,
        nf: Nullifier,
        rk: VerificationKey<SpendAuth>,
        nft: bool,
        b_d1: NoteValue,
        b_d2: NoteValue,
        b_sc: NoteValue,
        c_d1: NoteValue,
        cmb: ExtractedNoteCommitment,
        cmc: ExtractedNoteCommitment,
    ) -> Self {
        Instance {
            anchor,
            nf,
            rk,
            nft,
            b_d1,
            b_d2,
            b_sc,
            c_d1,
            cmb,
            cmc,
        }
    }

    pub fn to_halo2_instance(&self) -> [[vesta::Scalar; 11]; 1] {
        let mut instance = [vesta::Scalar::zero(); 11];

        instance[ANCHOR] = self.anchor.inner();
        instance[NF] = self.nf.0;

        let rk = pallas::Point::from_bytes(&self.rk.clone().into())
            .unwrap()
            .to_affine()
            .coordinates()
            .unwrap();

        instance[RK_X] = *rk.x();
        instance[RK_Y] = *rk.y();
        instance[NFT] = vesta::Scalar::from(self.nft);
        instance[B_D1] = vesta::Scalar::from(self.b_d1.inner());
        instance[B_D2] = vesta::Scalar::from(self.b_d2.inner());
        instance[B_SC] = vesta::Scalar::from(self.b_sc.inner());
        instance[C_D1] = vesta::Scalar::from(self.c_d1.inner());
        instance[CMB] = self.cmb.inner();
        instance[CMC] = self.cmc.inner();

        [instance]
    }
}

impl rustzeos::halo2::Instance for Instance{
    fn to_halo2_instance_vec(&self) -> Vec<Vec<vesta::Scalar>> {
        let instance = self.to_halo2_instance()[0].to_vec();
        let mut instances = Vec::new();
        instances.push(instance);
        instances
    }
}

#[cfg(test)]
// cargo test --package orchard --lib -- circuit::tests --nocapture
mod tests {
    use core::iter;

    use ff::Field;
    use halo2_proofs::{circuit::Value, dev::MockProver};
    use pasta_curves::pallas;
    use rand::{rngs::OsRng, RngCore};

    use super::{Circuit, Instance, K};
    use rustzeos::halo2::{Proof, ProvingKey, VerifyingKey, Instance as ConcreteInstance};
    use crate::{
        keys::SpendValidatingKey,
        note::{Note, NT_FT},
        tree::MerklePath,
        value::{NoteValue},
    };

    fn generate_circuit_instance<R: RngCore>(mut rng: R) -> (Circuit, Instance) {
        let (_, fvk, note_a) = Note::dummy(&mut rng, None, Some(NoteValue::from_raw(7)));

        let sender_address = note_a.recipient();
        let nk = *fvk.nk();
        let rivk = fvk.rivk(fvk.scope_for_address(&note_a.recipient()).unwrap());
        let nf_a = note_a.nullifier(&fvk);
        let ak: SpendValidatingKey = fvk.into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let note_b = Note::new(NT_FT, sender_address, NoteValue::from_raw(3), NoteValue::zero(), NoteValue::zero(), NoteValue::zero(), nf_a, &mut rng, [0; 512]);
        let note_c = Note::new(NT_FT, sender_address, NoteValue::from_raw(4), NoteValue::zero(), NoteValue::zero(), NoteValue::zero(), nf_a, &mut rng, [0; 512]);

        let path = MerklePath::dummy(&mut rng);
        let anchor = path.root(note_a.commitment().into());

        (
            Circuit {
                path: Value::known(path.auth_path()),
                pos: Value::known(path.position()),
                g_d_a: Value::known(sender_address.g_d()),
                pk_d_a: Value::known(*sender_address.pk_d()),
                d1_a: Value::known(note_a.d1()),
                d2_a: Value::known(note_a.d2()),
                rho_a: Value::known(note_a.rho()),
                psi_a: Value::known(note_a.rseed().psi(&note_a.rho())),
                rcm_a: Value::known(note_a.rseed().rcm(&note_a.rho())),
                cm_a: Value::known(note_a.commitment()),
                alpha: Value::known(alpha),
                ak: Value::known(ak),
                nk: Value::known(nk),
                rivk: Value::known(rivk),
                g_d_b: Value::known(note_b.recipient().g_d()),
                pk_d_b: Value::known(*note_b.recipient().pk_d()),
                d1_b: Value::known(note_b.d1()),
                d2_b: Value::known(note_b.d2()),
                sc_b: Value::known(note_b.sc()),
                rho_b: Value::known(nf_a),
                psi_b: Value::known(note_b.rseed().psi(&note_b.rho())),
                rcm_b: Value::known(note_b.rseed().rcm(&note_b.rho())),
                g_d_c: Value::known(sender_address.g_d()),
                pk_d_c: Value::known(*sender_address.pk_d()),
                d1_c: Value::known(note_c.d1()),
                psi_c: Value::known(note_c.rseed().psi(&note_c.rho())),
                rcm_c: Value::known(note_c.rseed().rcm(&note_c.rho())),
            },
            Instance {
                anchor: anchor,
                nf: nf_a,
                rk: rk,
                nft: false,
                b_d1: NoteValue::from_raw(0),
                b_d2: NoteValue::from_raw(0),
                b_sc: NoteValue::from_raw(0),
                c_d1: NoteValue::from_raw(0),
                cmb: note_b.commitment().into(),
                cmc: note_c.commitment().into(),
            },
        )
    }

    // TODO: recast as a proptest
    #[test]
    fn round_trip() {
        let mut rng = OsRng;

        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())//iter::once(()).chain(iter::once(()))
            .map(|()| generate_circuit_instance(&mut rng))
            .unzip();
        
        let vk = VerifyingKey::build(Circuit::default(), K);

        // serialize and deserialize vk back and forth
        let mut arr = Vec::new();
        vk.serialize(&mut arr);
        let vk = VerifyingKey::deserialize(&mut arr);
/*
        // Test that the pinned verification key (representing the circuit)
        // is as expected.
        {
            panic!("{:#?}", vk.vk.pinned());
            assert_eq!(
                format!("{:#?}\n", vk.vk.pinned()),
                include_str!("circuit_description").replace("\r\n", "\n")
            );
        }

        // Test that the proof size is as expected.
        let expected_proof_size = {
            let circuit_cost =
                halo2_proofs::dev::CircuitCost::<pasta_curves::vesta::Point, _>::measure(
                    K as usize,
                    &circuits[0],
                );
            assert_eq!(usize::from(circuit_cost.proof_size(1)), 4992);
            assert_eq!(usize::from(circuit_cost.proof_size(2)), 7264);
            usize::from(circuit_cost.proof_size(instances.len()))
        };
*/
        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
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

        let pk = ProvingKey::build(Circuit::default(), K);
        let proof = Proof::create(&pk, &circuits, &instances, &mut rng).unwrap();
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance_vec()).collect();
        assert!(proof.verify(&vk, &instances).is_ok());
        //assert_eq!(proof.0.len(), expected_proof_size);
    }

    #[test]
    fn serialized_proof_test_case() {
        use std::io::{Read, Write};

        let vk = VerifyingKey::build(Circuit::default(), K);

        fn write_test_case<W: Write>(
            mut w: W,
            instance: &Instance,
            proof: &Proof,
        ) -> std::io::Result<()> {
            w.write_all(&instance.anchor.to_bytes())?;
            w.write_all(&instance.nf.to_bytes())?;
            w.write_all(&<[u8; 32]>::from(instance.rk.clone()))?;
            w.write_all(&[instance.nft as u8; 1])?;
            w.write_all(&instance.b_d1.to_bytes())?;
            w.write_all(&instance.b_d2.to_bytes())?;
            w.write_all(&instance.b_sc.to_bytes())?;
            w.write_all(&instance.c_d1.to_bytes())?;
            w.write_all(&instance.cmb.to_bytes())?;
            w.write_all(&instance.cmc.to_bytes())?;

            w.write_all(proof.as_ref())?;
            Ok(())
        }

        fn read_test_case<R: Read>(mut r: R) -> std::io::Result<(Instance, Proof)> {
            let read_32_bytes = |r: &mut R| {
                let mut ret = [0u8; 32];
                r.read_exact(&mut ret).unwrap();
                ret
            };
            let read_8_bytes = |r: &mut R| {
                let mut ret = [0u8; 8];
                r.read_exact(&mut ret).unwrap();
                ret
            };
            let read_bool = |r: &mut R| {
                let mut byte = [0u8; 1];
                r.read_exact(&mut byte).unwrap();
                match byte {
                    [0] => false,
                    [1] => true,
                    _ => panic!("Unexpected non-boolean byte"),
                }
            };

            let anchor = crate::Anchor::from_bytes(read_32_bytes(&mut r)).unwrap();
            let nf = crate::note::Nullifier::from_bytes(&read_32_bytes(&mut r)).unwrap();
            let rk = read_32_bytes(&mut r).try_into().unwrap();
            let nft = read_bool(&mut r);
            let b_d1 = NoteValue::from_bytes(read_8_bytes(&mut r).try_into().unwrap());
            let b_d2 = NoteValue::from_bytes(read_8_bytes(&mut r).try_into().unwrap());
            let b_sc = NoteValue::from_bytes(read_8_bytes(&mut r).try_into().unwrap());
            let c_d1 = NoteValue::from_bytes(read_8_bytes(&mut r).try_into().unwrap());
            let cmb = crate::note::ExtractedNoteCommitment::from_bytes(&read_32_bytes(&mut r)).unwrap();
            let cmc = crate::note::ExtractedNoteCommitment::from_bytes(&read_32_bytes(&mut r)).unwrap();
            
            let instance = Instance::from_parts(anchor, nf, rk, nft, b_d1, b_d2, b_sc, c_d1, cmb, cmc);

            let mut proof_bytes = vec![];
            r.read_to_end(&mut proof_bytes)?;
            let proof = Proof::new(proof_bytes);

            Ok((instance, proof))
        }

        //if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
        if true {
            let create_proof = || -> std::io::Result<()> {
                let mut rng = OsRng;

                let (circuit, instance) = generate_circuit_instance(OsRng);
                let instances = &[instance.clone()];

                let pk = ProvingKey::build(Circuit::default(), K);
                let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();
                let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance_vec()).collect();
                assert!(proof.verify(&vk, &instances).is_ok());

                let file = std::fs::File::create("src/circuit_proof_test_case.bin")?;
                write_test_case(file, &instance, &proof)
            };
            create_proof().expect("should be able to write new proof");
        }

        // Parse the hardcoded proof test case.
        let (instance, proof) = {
            let test_case_bytes = include_bytes!("circuit_proof_test_case.bin");
            read_test_case(&test_case_bytes[..]).expect("proof must be valid")
        };
        //assert_eq!(proof.0.len(), 4992);

        let instances = &[instance];
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance_vec()).collect();
        assert!(proof.verify(&vk, &instances).is_ok());
    }

    // cargo test --features dev-graph --package zeos-orchard --lib -- circuit::tests::print_action_circuit --exact --nocapture
    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_action_circuit() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("action-circuit-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("ZEOS Action Circuit", ("sans-serif", 60))
            .unwrap();

        let circuit = Circuit {
            path: Value::unknown(),
            pos: Value::unknown(),
            g_d_a: Value::unknown(),
            pk_d_a: Value::unknown(),
            d1_a: Value::unknown(),
            d2_a: Value::unknown(),
            rho_a: Value::unknown(),
            psi_a: Value::unknown(),
            rcm_a: Value::unknown(),
            cm_a: Value::unknown(),
            alpha: Value::unknown(),
            ak: Value::unknown(),
            nk: Value::unknown(),
            rivk: Value::unknown(),
            g_d_b: Value::unknown(),
            pk_d_b: Value::unknown(),
            d1_b: Value::unknown(),
            d2_b: Value::unknown(),
            sc_b: Value::unknown(),
            rho_b: Value::unknown(),
            psi_b: Value::unknown(),
            rcm_b: Value::unknown(),
            g_d_c: Value::unknown(),
            pk_d_c: Value::unknown(),
            d1_c: Value::unknown(),
            psi_c: Value::unknown(),
            rcm_c: Value::unknown(),
        };
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(false)
            .view_height(0..(1 << 11))
            .render(K, &circuit, &root)
            .unwrap();
    }
}
