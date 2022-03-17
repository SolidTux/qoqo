// Copyright © 2021 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use crate::operations::single_qubit_gate_operations::*;
use crate::prelude::*;
use crate::Circuit;
use crate::RoqoqoError;
use ndarray::{array, Array2};
use num_complex::Complex64;
use qoqo_calculator::{CalculatorComplex, CalculatorFloat};
#[cfg(feature = "overrotate")]
use rand_distr::{Distribution, Normal};
use std::convert::TryFrom;
use std::f64::consts::PI;

/// The KAK decomposition of a two-qubit gate.
///
/// Each two-qubit gate can be described by a KAK decomposition (<http://arxiv.org/abs/quant-ph/0507171>).
///
/// A two qubit gate is decomposed into four single qubit gates, one for each qubit acting before and after applying the
/// entangling operation based on the k_vector:  
///
/// U(k_vector) = exp(i (k_vector(0) XX + k_vector(1) YY + k_vector(2) ZZ))
///
/// This struct contains all information on the KAK decomposition of a two qubit gate.
#[derive(Debug, Clone, PartialEq)]
pub struct KakDecomposition {
    /// Global phase of KAK decomposition
    pub global_phase: CalculatorFloat,
    /// Three component vector of the KAK decomposition
    pub k_vector: [CalculatorFloat; 3],
    /// Circuit including operations acting on control and target qubits before two-qubit entangling
    pub circuit_before: Option<Circuit>,
    /// Circuit including operations acting on control and target qubits after two-qubit entangling
    pub circuit_after: Option<Circuit>,
}

/// Implements the CNOT controlled not gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 0 & 1 \\\\
/// 0 & 0 & 1 & 0
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct CNOT {
    /// The index of the most significant qubit in the unitary representation. Here, the qubit that controls the application of NOT on the target qubit.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. Here, the qubit NOT is applied to.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_CNOT: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "CNOT",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for CNOT {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for CNOT {
    /// Returns [KakDecomposition] of the  gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateY::new(self.control, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateX::new(self.target, CalculatorFloat::FRAC_PI_2);

        let mut circuit_a = Circuit::new();
        circuit_a += RotateY::new(self.control, CalculatorFloat::FRAC_PI_2 * (-1.0));

        KakDecomposition {
            global_phase: CalculatorFloat::FRAC_PI_4,
            k_vector: [
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// Implements the SWAP gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 0 & 1 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct SWAP {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_SWAP: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "SWAP",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for SWAP {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for SWAP {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::from((-1.0) * PI / 4.0),
            k_vector: [
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::FRAC_PI_4,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The ISwap gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 0 & i & 0 \\\\
/// 0 & i & 0 & 0 \\\\
/// 0 & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct ISwap {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_ISwap: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "ISwap",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for ISwap {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for ISwap {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The fermionic SWAP gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 0 & 1 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 0 & -1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct FSwap {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_FSwap: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "FSwap",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for FSwap {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(-1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for FSwap {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2 * (-1.0));
        circuit_b += RotateZ::new(self.target, CalculatorFloat::FRAC_PI_2 * (-1.0));

        KakDecomposition {
            global_phase: CalculatorFloat::FRAC_PI_2 * (-1.0),
            k_vector: [
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::FRAC_PI_4,
                CalculatorFloat::ZERO,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: None,
        }
    }
}

/// The square root ISwap gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & \frac{1}{\sqrt{2}} & \frac{i}{\sqrt{2}} & 0 \\\\
/// 0 & \frac{i}{\sqrt{2}} & \frac{1}{\sqrt{2}} & 0 \\\\
/// 0 & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct SqrtISwap {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_SqrtISwap: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "SqrtISwap",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for SqrtISwap {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let f: f64 = 1.0 / ((2.0_f64).sqrt());
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, f),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, f),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for SqrtISwap {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                CalculatorFloat::from(PI / 8.0),
                CalculatorFloat::from(PI / 8.0),
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The inverse square root ISwap gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & \frac{1}{\sqrt{2}} & \frac{-i}{\sqrt{2}} & 0 \\\\
/// 0 & \frac{-i}{\sqrt{2}} & \frac{1}{\sqrt{2}} & 0 \\\\
/// 0 & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct InvSqrtISwap {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_InvSqrtISwap: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "InvSqrtISwap",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for InvSqrtISwap {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let f: f64 = 1.0 / ((2.0_f64).sqrt());
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, (-1.0) * f),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * f),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for InvSqrtISwap {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                CalculatorFloat::from((-1.0) * PI / 8.0),
                CalculatorFloat::from((-1.0) * PI / 8.0),
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The XY gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & \cos(\theta/2) & i \sin(\theta/2) & 0 \\\\
/// 0 & i \sin(\theta/2) & \cos(\theta/2) & 0 \\\\
/// 0 & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
    roqoqo_derive::Rotate,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct XY {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The rotation angle $\theta$.
    theta: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_XY: &[&str; 5] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Rotation",
    "XY",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for XY {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let c: f64 = (f64::try_from(self.theta.clone())? / 2.0).cos();
        let s: f64 = (f64::try_from(self.theta.clone())? / 2.0).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, s),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, s),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for XY {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                self.theta.clone() / 4.0,
                self.theta.clone() / 4.0,
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// Implements the controlled-PhaseShift gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 1 & 0 \\\\
/// 0 & 0 & 0 & e^{i \theta}
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
    roqoqo_derive::Rotate,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct ControlledPhaseShift {
    /// The index of the most significant qubit in the unitary representation. Here, the qubit that controls the application of the phase-shift on the target qubit.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. Here, the qubit phase-shift is applied to.
    target: usize,
    /// The rotation angle $\theta$.
    theta: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_ControlledPhaseShift: &[&str; 5] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Rotation",
    "ControlledPhaseShift",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for ControlledPhaseShift {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        // exp(i*x) = cos(x)+i*sin(x)
        let c: f64 = (f64::try_from(self.theta.clone())?).cos();
        let s: f64 = (f64::try_from(self.theta.clone())?).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(c, s)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for ControlledPhaseShift {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, self.theta.clone() / 2.0);
        circuit_b += RotateZ::new(self.target, self.theta.clone() / 2.0);

        KakDecomposition {
            global_phase: self.theta.clone() / 4.0,
            k_vector: [
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
                self.theta.clone() / 4.0,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: None,
        }
    }
}

/// The controlled-PauliY gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 0 & -i \\\\
/// 0 & 0 & i & 0
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct ControlledPauliY {
    /// The index of the most significant qubit in the unitary representation. Here, the qubit that controls the application of PauliY gate on the target qubit.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. Here, the qubit PauliY is applied to.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_ControlledPauliY: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "ControlledPauliY",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for ControlledPauliY {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, -1.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 1.0),
                Complex64::new(0.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for ControlledPauliY {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateY::new(self.target, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateX::new(self.target, CalculatorFloat::FRAC_PI_2);

        let mut circuit_a = Circuit::new();
        circuit_a += RotateX::new(self.target, CalculatorFloat::FRAC_PI_2 * (-1.0));

        KakDecomposition {
            global_phase: CalculatorFloat::FRAC_PI_4,
            k_vector: [
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
                CalculatorFloat::FRAC_PI_4,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// The controlled-PauliZ gate.
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 1 & 0 \\\\
/// 0 & 0 & 0 & -1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct ControlledPauliZ {
    /// The index of the most significant qubit in the unitary representation. Here, the qubit that controls the application of PauliZ gate on the target qubit.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. Here, the qubit PauliZ is applied to.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_ControlledPauliZ: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "ControlledPauliZ",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for ControlledPauliZ {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(-1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for ControlledPauliZ {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateZ::new(self.target, CalculatorFloat::FRAC_PI_2);

        KakDecomposition {
            global_phase: CalculatorFloat::FRAC_PI_4,
            k_vector: [
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
                CalculatorFloat::FRAC_PI_4,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: None,
        }
    }
}

/// The fixed phase MolmerSorensen XX gate.
///
/// <http://arxiv.org/abs/1705.02771>
///
/// $$
/// U = \frac{1}{\sqrt{2}} \begin{pmatrix}
/// 1 & 0 & 0 & -i \\\\
/// 0 &1 & -i & 0 \\\\
/// 0 & -i & 1 & 0 \\\\
/// -i & 0 & 0 & 1
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct MolmerSorensenXX {
    /// The index of the most significant qubit in the unitary representation. The gate is symmetric under the exchange of qubits.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. The gate is symmetric under the exchange of qubits.
    target: usize,
}

#[allow(non_upper_case_globals)]
const TAGS_MolmerSorensenXX: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "MolmerSorensenXX",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for MolmerSorensenXX {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed (here, not possible).
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let f: f64 = 1.0 / ((2.0_f64).sqrt());
        Ok(array![
            [
                Complex64::new(f, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * f)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, (-1.0) * f),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * f),
                Complex64::new(f, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, (-1.0) * f),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(f, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for MolmerSorensenXX {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                CalculatorFloat::from((-1.0) * PI / 4.0),
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The variable-angle MolmerSorensen XX gate.
///
/// $$
/// U = \begin{pmatrix}
/// \cos(\theta/2) & 0 & 0 & -i \sin(\theta/2) \\\\
/// 0 & \cos(\theta/2) & -i \sin(\theta/2) & 0 \\\\
/// 0 & -i \sin(\theta/2) & \cos(\theta/2) & 0 \\\\
/// -i \sin(\theta/2) & 0 & 0 & \cos(\theta/2)
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
    roqoqo_derive::Rotate,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct VariableMSXX {
    /// The index of the most significant qubit in the unitary representation. The gate is symmetric under the exchange of qubits.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. The gate is symmetric under the exchange of qubits.
    target: usize,
    /// The rotation angle $\theta$.
    theta: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_VariableMSXX: &[&str; 5] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Rotation",
    "VariableMSXX",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for VariableMSXX {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let c: f64 = (f64::try_from(self.theta.clone())? / 2.0).cos();
        let s: f64 = (f64::try_from(self.theta.clone())? / 2.0).sin();
        Ok(array![
            [
                Complex64::new(c, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * s)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, (-1.0) * s),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * s),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, (-1.0) * s),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(c, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for VariableMSXX {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                self.theta.clone() * (-1.0 / 2.0),
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The Givens rotation interaction gate in big endian notation: $e^{-\mathrm{i} \theta (X_c Y_t - Y_c X_t)}\cdot e^{-i \phi Z_t/2}$.
///
/// Where $X_c$ is the Pauli matrix $\sigma^x$ acting on the control qubit, $Y_t$ is the Pauli matrix $\sigma^y$ acting on the target qubit,
/// and $Z_t$ is the Pauli matrix $\sigma^z$ acting on the target qubit.
///
/// The unitary matrix representation is:
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & \cos(\theta) \cdot e^{i \phi} & \sin(\theta)  & 0 \\\\
/// 0 & -\sin(\theta) \cdot e^{i \phi} & \cos(\theta)  & 0 \\\\
/// 0 & 0 & 0 & e^{i \phi}
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
    roqoqo_derive::Rotate,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct GivensRotation {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The rotation angle $\theta$.
    theta: CalculatorFloat,
    /// The phase $\phi$ of the rotation.
    phi: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_GivensRotation: &[&str; 5] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Rotation",
    "GivensRotation",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for GivensRotation {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let ct: f64 = (f64::try_from(self.theta.clone())?).cos();
        let st: f64 = (f64::try_from(self.theta.clone())?).sin();
        // exp(i*phi) = cos(phi)+i*sin(phi)
        let cp: f64 = (f64::try_from(self.phi.clone())?).cos();
        let sp: f64 = (f64::try_from(self.phi.clone())?).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(ct * cp, ct * sp),
                Complex64::new(st, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * st * cp, (-1.0) * st * sp),
                Complex64::new(ct, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cp, sp)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for GivensRotation {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.target, self.phi.clone() + (PI / 2.0));

        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.target, CalculatorFloat::FRAC_PI_2 * (-1.0));

        KakDecomposition {
            global_phase: self.phi.clone() / 2.0,
            k_vector: [
                self.theta.clone() / 2.0,
                self.theta.clone() / 2.0,
                CalculatorFloat::ZERO,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// The Givens rotation interaction gate in little endian notation: $e^{-\mathrm{i} \theta (X_c Y_t -Y_c  X_t)}\cdot e^{-i \phi Z_c/2}$.
///
/// Where $X_c$ is the Pauli matrix $\sigma^x$ acting on the control qubit, $Y_t$ is the Pauli matrix $\sigma^y$ acting on the target qubit,
/// and $Z_c$ is the Pauli matrix $\sigma^z$ acting on the control qubit.
///
/// The unitary matrix representation is:
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & \cos(\theta) & \sin(\theta)  & 0 \\\\
/// 0 & -\sin(\theta) \cdot e^{i \phi} & \cos(\theta) \cdot e^{i \phi}  & 0 \\\\
/// 0 & 0 & 0 & e^{i \phi}
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
    roqoqo_derive::Rotate,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct GivensRotationLittleEndian {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The rotation angle $\theta$.
    theta: CalculatorFloat,
    /// The phase $\phi$ of the rotation.
    phi: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_GivensRotationLittleEndian: &[&str; 5] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Rotation",
    "GivensRotationLittleEndian",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for GivensRotationLittleEndian {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let ct: f64 = (f64::try_from(self.theta.clone())?).cos();
        let st: f64 = (f64::try_from(self.theta.clone())?).sin();
        // exp(i*phi) = cos(phi)+i*sin(phi)
        let cp: f64 = (f64::try_from(self.phi.clone())?).cos();
        let sp: f64 = (f64::try_from(self.phi.clone())?).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(ct, 0.0),
                Complex64::new(st, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * st * cp, (-1.0) * st * sp),
                Complex64::new(ct * cp, ct * sp),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cp, sp)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for GivensRotationLittleEndian {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2 * (-1.0));

        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.control, self.phi.clone() + (PI / 2.0));

        KakDecomposition {
            global_phase: self.phi.clone() / 2.0,
            k_vector: [
                self.theta.clone() / 2.0,
                self.theta.clone() / 2.0,
                CalculatorFloat::ZERO,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// The qubit simulation (Qsim) gate.
///
/// $$
/// U = \begin{pmatrix}
/// \cos(x-y) \cdot e^{-i z} & 0 & 0 & -i\sin(x-y)\cdot e^{-i z}\\\\
/// 0 & -i \sin(x+y)\cdot e^{i z} & \cos(x+y)\cdot e^{i z} & 0 \\\\
/// 0 & \cos(x+y)\cdot e^{i z}& -i \sin(x+y)\cdot e^{i z} & 0 \\\\
/// -\sin(x-y)\cdot e^{-i z} & 0 & 0 & \cos(x-y)\cdot e^{-i z}
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct Qsim {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The prefactor of the XX interaction.
    x: CalculatorFloat,
    /// The prefactor of the YY interaction.
    y: CalculatorFloat,
    /// The prefactor of the ZZ interaction.
    z: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_Qsim: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Qsim",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for Qsim {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let x: f64 = f64::try_from(self.x.clone())?;
        let y: f64 = f64::try_from(self.y.clone())?;
        let z: f64 = f64::try_from(self.z.clone())?;

        let cm: f64 = (x - y).cos();
        let cp: f64 = (x + y).cos();
        let sm: f64 = (x - y).sin();
        let sp: f64 = (x + y).sin();

        // exp(i*z) = cos(z) + i*sin(z)
        // exp(-i*z) = cos(z) - i*sin(z)
        let cz: f64 = z.cos();
        let sz: f64 = z.sin();

        Ok(array![
            [
                Complex64::new(cm * cz, (-1.0) * cm * sz),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * sm * sz, (-1.0) * sm * cz)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(sp * sz, (-1.0) * sp * cz),
                Complex64::new(cp * cz, cp * sz),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(cp * cz, cp * sz),
                Complex64::new(sp * sz, (-1.0) * sp * cz),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new((-1.0) * sm * sz, (-1.0) * sm * cz),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cm * cz, (-1.0) * cm * sz)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for Qsim {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::from(-1.0 * PI / 4.0),
            k_vector: [
                self.x.clone() * (-1.0) + PI / 4.0,
                self.y.clone() * (-1.0) + PI / 4.0,
                self.z.clone() * (-1.0) + PI / 4.0,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The fermionic qubit simulation (Fsim) gate.
///
/// $$
/// U = \begin{pmatrix}
/// \cos(\Delta) & 0 & 0 & i \sin(\Delta) \\\\
/// 0 & -i \sin(t) & \cos(t) & 0 \\\\
/// 0 & \cos(t) & -i \sin(t) & 0 \\\\
/// -\sin(\Delta) \cdot e^{-i U} & 0 & 0 & -\cos(\Delta) \cdot e^{-i U}
/// \end{pmatrix}
/// $$
///
/// # Note
/// The qubits have to be adjacent, i.e., :math:`|i-j|=1` has to hold. This is the only case
/// in which the gate is valid as a two-qubit gate (due to the Jordan-Wigner transformation).
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct Fsim {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The hopping strength.
    t: CalculatorFloat,
    /// The interaction strength.
    u: CalculatorFloat,
    /// The Bogoliubov interaction strength $\Delta$.
    delta: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_Fsim: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Fsim",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for Fsim {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let t: f64 = f64::try_from(self.t.clone())?;
        let u: f64 = f64::try_from(self.u.clone())?;
        let d: f64 = f64::try_from(self.delta.clone())?;

        Ok(array![
            [
                Complex64::new(d.cos(), 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, d.sin())
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * t.sin()),
                Complex64::new(t.cos(), 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(t.cos(), 0.0),
                Complex64::new(0.0, (-1.0) * t.sin()),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new((-1.0) * d.sin() * u.sin(), (-1.0) * d.sin() * u.cos()),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * d.cos() * u.cos(), d.cos() * u.sin())
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for Fsim {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let theta = self.u.clone() / (-2.0) - PI / 2.0;
        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.control, theta.clone());
        circuit_a += RotateZ::new(self.target, theta);

        KakDecomposition {
            global_phase: self.u.clone() / (-4.0) - PI / 2.0,
            k_vector: [
                (self.t.clone() / (-2.0) + self.delta.clone() / 2.0 + PI / 4.0),
                (self.t.clone() / (-2.0) - self.delta.clone() / 2.0 + PI / 4.0),
                self.u.clone() / (-4.0),
            ],
            circuit_before: None,
            circuit_after: Some(circuit_a),
        }
    }
}

/// The generalized, anisotropic XYZ Heisenberg interaction between spins.
///
/// $$
/// e^{-\mathrm{i} (x \cdot X_c X_t + y \cdot Y_c Y_t + z \cdot Z_c Z_t)}
/// $$
///
/// Where x, y, z are prefactors of the $X_c X_t$, $Y_c Y_t$, $Z_c Z_t$ Pauliproducts acting on control and target qubit,
/// with $XX \equiv \sigma_x \sigma_x$, $YY \equiv \sigma_y \sigma_y$ and $ZZ \equiv \sigma_z \sigma_z$.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct SpinInteraction {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The prefactor of the XX interaction.
    x: CalculatorFloat,
    /// The prefactor of the YY interaction.
    y: CalculatorFloat,
    /// The prefactor of the ZZ interaction.
    z: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_SpinInteraction: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "SpinInteraction",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for SpinInteraction {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let x: f64 = f64::try_from(self.x.clone())?;
        let y: f64 = f64::try_from(self.y.clone())?;
        let z: f64 = f64::try_from(self.z.clone())?;

        let cm: f64 = (x - y).cos();
        let cp: f64 = (x + y).cos();
        let sm: f64 = (x - y).sin();
        let sp: f64 = (x + y).sin();

        // exp(i*z) = cos(z) + i*sin(z)
        // exp(-i*z) = cos(z) - i*sin(z)
        let cz: f64 = z.cos();
        let sz: f64 = z.sin();

        Ok(array![
            [
                Complex64::new(cm * cz, (-1.0) * cm * sz),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * sm * sz, (-1.0) * sm * cz)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(cp * cz, cp * sz),
                Complex64::new(sp * sz, (-1.0) * sp * cz),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(sp * sz, (-1.0) * sp * cz),
                Complex64::new(cp * cz, cp * sz),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new((-1.0) * sm * sz, (-1.0) * sm * cz),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cm * cz, (-1.0) * cm * sz)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for SpinInteraction {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                self.x.clone() * (-1.0),
                self.y.clone() * (-1.0),
                self.z.clone() * (-1.0),
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The Bogoliubov DeGennes interaction gate.
///
/// $$
/// e^{-\mathrm{i} Re(\Delta) (X_c X_t - Y_c Y_t)/2 + Im(\Delta) (X_c Y_t+Y_c X_t)/2}
/// $$
///
/// Where $X_c$ is the Pauli matrix $\sigma^x$ acting on the control qubit, and $Y_t$ is the Pauli matrix $\sigma^y$ acting on the target qubit.
///
/// The unitary matrix representation is:
/// $$
/// U = \begin{pmatrix}
/// \cos(|\Delta|) & 0 & 0 & \mathrm{i} \sin(|\Delta|) e^{\mathrm{i} \cdot \mathrm{angle}(\Delta)} \\\\
/// 0 & 1 & 0 & 0 \\\\
/// 0 & 0 & 1 & 0 \\\\
///  \mathrm{i} \sin(|\Delta|) e^{-\mathrm{i} \cdot \mathrm{angle}(\Delta)} & 0 & 0 & \cos(|\Delta|)
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct Bogoliubov {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The real part of the complex Bogoliubov interaction strength $Re(\Delta)$
    delta_real: CalculatorFloat,
    /// The imaginary part of the complex Bogoliubov interaction strength $Im(\Delta)$
    delta_imag: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_Bogoliubov: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "Bogoliubov",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for Bogoliubov {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let dr: f64 = f64::try_from(self.delta_real.clone())?;
        let di: f64 = f64::try_from(self.delta_imag.clone())?;
        let delta: Complex64 = Complex64::new(dr, di);
        let da: f64 = delta.norm(); //absolute value of delta
        let dp: f64 = delta.arg(); // phase of delta
        Ok(array![
            [
                Complex64::new(da.cos(), 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new((-1.0) * da.sin() * dp.sin(), da.sin() * dp.cos())
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(da.sin() * dp.sin(), da.sin() * dp.cos()),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(da.cos(), 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for Bogoliubov {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let dr = self.delta_real.clone();
        let di = self.delta_imag.clone();
        let delta: CalculatorComplex = CalculatorComplex::new(dr, di);

        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.target, delta.arg());

        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.target, delta.arg() * (-1.0));

        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                delta.norm() / (2.0),
                delta.norm() / (-2.0),
                CalculatorFloat::ZERO,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// The transversal interaction gate.
///
/// $$
/// e^{-\mathrm{i} \theta (X_c X_t + Y_c Y_t)} = e^{-\mathrm{i} \theta (\sigma^+_c \sigma^-_t + \sigma^-_c \sigma^+_t)}
/// $$
/// Where $X_c$ is the Pauli matrix $\sigma^x$ acting on the control qubit, and $Y_t$ is the Pauli matrix $\sigma^y$ acting on the target qubit.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct PMInteraction {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The strength of the rotation $\theta$.
    t: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_PMInteraction: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "PMInteraction",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for PMInteraction {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let c: f64 = (f64::try_from(self.t.clone())?).cos();
        let s: f64 = (f64::try_from(self.t.clone())?).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, (-1.0) * s),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, (-1.0) * s),
                Complex64::new(c, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for PMInteraction {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [
                self.t.clone() / (-2.0),
                self.t.clone() / (-2.0),
                CalculatorFloat::ZERO,
            ],
            circuit_before: None,
            circuit_after: None,
        }
    }
}

/// The complex hopping gate.
///
/// $$
/// e^{-\mathrm{i} \left[ Re(\theta) \cdot (X_c X_t + Y_c Y_t) - Im(\theta) \cdot (X_c Y_t - Y_c X_t) \right] }
/// $$
/// Where $X_c$ is the Pauli matrix $\sigma^x$ acting on the control qubit, and $Y_t$ is the Pauli matrix $\sigma^y$ acting on the target qubit.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct ComplexPMInteraction {
    /// The index of the most significant qubit in the unitary representation.
    control: usize,
    /// The index of the least significant qubit in the unitary representation.
    target: usize,
    /// The real part of the strength of the rotation $Re(\theta)$.
    t_real: CalculatorFloat,
    /// The imaginary part of the strength of the rotation $Im(\theta)$.
    t_imag: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_ComplexPMInteraction: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "ComplexPMInteraction",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for ComplexPMInteraction {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        let tr: f64 = f64::try_from(self.t_real.clone())?;
        let ti: f64 = f64::try_from(self.t_imag.clone())?;
        let t: Complex64 = Complex64::new(tr, ti);
        let tn: f64 = t.norm(); //absolute value of delta
        let ta: f64 = t.arg(); // phase of delta
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(tn.cos(), 0.0),
                Complex64::new((-1.0) * tn.sin() * ta.sin(), (-1.0) * tn.sin() * ta.cos()),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(tn.sin() * ta.sin(), (-1.0) * tn.sin() * ta.cos()),
                Complex64::new(tn.cos(), 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for ComplexPMInteraction {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let tr = self.t_real.clone();
        let ti = self.t_imag.clone();
        let t: CalculatorComplex = CalculatorComplex::new(tr, ti);

        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.target, t.arg());

        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.target, t.arg() * (-1.0));

        KakDecomposition {
            global_phase: CalculatorFloat::ZERO,
            k_vector: [t.norm() / (-2.0), t.norm() / (-2.0), CalculatorFloat::ZERO],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}

/// Implements the phased-shifted controlled-Z gate.
///
/// Modified, i.e. phase-shifted ControlledPauliZ two-qubit gate (`<https://arxiv.org/pdf/1908.06101.pdf eq.(1)>`).
/// The unitary matrix representation is:
///
/// $$
/// U = \begin{pmatrix}
/// 1 & 0 & 0 & 0 \\\\
/// 0 & e^{i \phi} & 0 & 0 \\\\
/// 0 & 0 & e^{i \phi} & 0 \\\\
/// 0 & 0 & 0 & e^{i (2\cdot\phi - \pi)}
/// \end{pmatrix}
/// $$
///
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    roqoqo_derive::InvolveQubits,
    roqoqo_derive::Operate,
    roqoqo_derive::Substitute,
    roqoqo_derive::OperateTwoQubit,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
pub struct PhaseShiftedControlledZ {
    /// The index of the most significant qubit in the unitary representation. Here, the qubit that controls the application of the phase-shift on the target qubit.
    control: usize,
    /// The index of the least significant qubit in the unitary representation. Here, the qubit phase-shift is applied to.
    target: usize,
    /// The single qubit phase $\phi$.
    phi: CalculatorFloat,
}

#[allow(non_upper_case_globals)]
const TAGS_PhaseShiftedControlledZ: &[&str; 4] = &[
    "Operation",
    "GateOperation",
    "TwoQubitGateOperation",
    "PhaseShiftedControlledZ",
];

/// Trait for all Operations acting with a unitary gate on a set of qubits.
impl OperateGate for PhaseShiftedControlledZ {
    /// Returns unitary matrix of the gate.
    ///
    /// # Returns
    ///
    /// * `Ok(Array2<Complex64>)` - The unitary matrix representation of the gate.
    /// * `Err(RoqoqoError)` - The conversion of parameters to f64 failed.
    fn unitary_matrix(&self) -> Result<Array2<Complex64>, RoqoqoError> {
        // exp(i*x) = cos(x)+i*sin(x)
        let phi: f64 = f64::try_from(self.phi.clone())?;
        let cos: f64 = phi.cos();
        let sin: f64 = phi.sin();
        let cos2: f64 = (2.0 * phi - PI).cos();
        let sin2: f64 = (2.0 * phi - PI).sin();
        Ok(array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(cos, sin),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cos, sin),
                Complex64::new(0.0, 0.0)
            ],
            [
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(cos2, sin2)
            ],
        ])
    }
}

/// Trait for all gate operations acting on exactly two qubits.
impl OperateTwoQubitGate for PhaseShiftedControlledZ {
    /// Returns [KakDecomposition] of the gate.
    ///
    /// # Returns
    ///
    /// * struct `KakDecomposition { global_phase, k_vector, circuit_before, circuit_after }`
    fn kak_decomposition(&self) -> KakDecomposition {
        let mut circuit_b = Circuit::new();
        circuit_b += RotateZ::new(self.control, CalculatorFloat::FRAC_PI_2);
        circuit_b += RotateZ::new(self.target, CalculatorFloat::FRAC_PI_2);

        let mut circuit_a = Circuit::new();
        circuit_a += RotateZ::new(self.control, self.phi.clone());
        circuit_a += RotateZ::new(self.target, self.phi.clone());

        let g: CalculatorFloat = CalculatorFloat::FRAC_PI_4 + self.phi.clone();
        KakDecomposition {
            global_phase: g,
            k_vector: [
                CalculatorFloat::ZERO,
                CalculatorFloat::ZERO,
                CalculatorFloat::FRAC_PI_4,
            ],
            circuit_before: Some(circuit_b),
            circuit_after: Some(circuit_a),
        }
    }
}
