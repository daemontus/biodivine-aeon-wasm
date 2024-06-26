// You production code that imports aeon_wasm through NPM, you can import from "aeon-wasm" directly
// instead of using the relative path.

import init, {
    Conversions,
    BooleanNetworkModel,
    ComputationResult,
    check_update_function
} from "../pkg/aeon_wasm.js"

/*
    Just some constants that we will use.
 */
const SBML_MODEL = `
<?xml version='1.0' encoding='UTF-8' standalone='no'?><sbml xmlns="http://www.sbml.org/sbml/level3/version1/core" layout:required="false" level="3" qual:required="true" xmlns:layout="http://www.sbml.org/sbml/level3/version1/layout/version1" version="1" xmlns:qual="http://www.sbml.org/sbml/level3/version1/qual/version1"><model><qual:listOfQualitativeSpecies xmlns:qual="http://www.sbml.org/sbml/level3/version1/qual/version1"><qual:qualitativeSpecies qual:maxLevel="1" qual:constant="false" qual:name="v_Coup_fti" qual:id="v_Coup_fti"/><qual:qualitativeSpecies qual:maxLevel="1" qual:constant="false" qual:name="v_Emx2" qual:id="v_Emx2"/><qual:qualitativeSpecies qual:maxLevel="1" qual:constant="false" qual:name="v_Fgf8" qual:id="v_Fgf8"/><qual:qualitativeSpecies qual:maxLevel="1" qual:constant="false" qual:name="v_Pax6" qual:id="v_Pax6"/><qual:qualitativeSpecies qual:maxLevel="1" qual:constant="false" qual:name="v_Sp8" qual:id="v_Sp8"/></qual:listOfQualitativeSpecies><qual:listOfTransitions xmlns:qual="http://www.sbml.org/sbml/level3/version1/qual/version1"><qual:transition qual:id="tr_v_Coup_fti"><qual:listOfInputs><qual:input qual:qualitativeSpecies="v_Fgf8" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Coup_fti_in_v_Fgf8" essential="true"/><qual:input qual:qualitativeSpecies="v_Sp8" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Coup_fti_in_v_Sp8" essential="true"/></qual:listOfInputs><qual:listOfOutputs><qual:output qual:qualitativeSpecies="v_Coup_fti" qual:transitionEffect="assignmentLevel" qual:id="tr_v_Coup_fti_out"/></qual:listOfOutputs><qual:listOfFunctionTerms><qual:defaultTerm qual:resultLevel="0"/><qual:functionTerm qual:resultLevel="1"><math xmlns="http://www.w3.org/1998/Math/MathML"><apply><or/><apply><not/><apply><or/><apply><eq/><ci>v_Fgf8</ci><cn type="integer">1</cn></apply><apply><eq/><ci>v_Sp8</ci><cn type="integer">1</cn></apply></apply></apply><apply><not/><apply><or/><apply><eq/><ci>v_Sp8</ci><cn type="integer">1</cn></apply><apply><eq/><ci>v_Fgf8</ci><cn type="integer">1</cn></apply></apply></apply></apply></math></qual:functionTerm></qual:listOfFunctionTerms></qual:transition><qual:transition qual:id="tr_v_Emx2"><qual:listOfInputs><qual:input qual:qualitativeSpecies="v_Coup_fti" qual:transitionEffect="none" qual:sign="positive" qual:id="tr_v_Emx2_in_v_Coup_fti" essential="true"/><qual:input qual:qualitativeSpecies="v_Fgf8" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Emx2_in_v_Fgf8" essential="true"/><qual:input qual:qualitativeSpecies="v_Pax6" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Emx2_in_v_Pax6" essential="true"/><qual:input qual:qualitativeSpecies="v_Sp8" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Emx2_in_v_Sp8" essential="true"/></qual:listOfInputs><qual:listOfOutputs><qual:output qual:qualitativeSpecies="v_Emx2" qual:transitionEffect="assignmentLevel" qual:id="tr_v_Emx2_out"/></qual:listOfOutputs><qual:listOfFunctionTerms><qual:defaultTerm qual:resultLevel="0"/><qual:functionTerm qual:resultLevel="1"><math xmlns="http://www.w3.org/1998/Math/MathML"><apply><and/><apply><eq/><ci>v_Coup_fti</ci><cn type="integer">1</cn></apply><apply><not/><apply><or/><apply><or/><apply><eq/><ci>v_Fgf8</ci><cn type="integer">1</cn></apply><apply><eq/><ci>v_Sp8</ci><cn type="integer">1</cn></apply></apply><apply><eq/><ci>v_Pax6</ci><cn type="integer">1</cn></apply></apply></apply></apply></math></qual:functionTerm></qual:listOfFunctionTerms></qual:transition><qual:transition qual:id="tr_v_Fgf8"><qual:listOfInputs><qual:input qual:qualitativeSpecies="v_Emx2" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Fgf8_in_v_Emx2" essential="true"/><qual:input qual:qualitativeSpecies="v_Fgf8" qual:transitionEffect="none" qual:sign="positive" qual:id="tr_v_Fgf8_in_v_Fgf8" essential="true"/><qual:input qual:qualitativeSpecies="v_Sp8" qual:transitionEffect="none" qual:sign="positive" qual:id="tr_v_Fgf8_in_v_Sp8" essential="true"/></qual:listOfInputs><qual:listOfOutputs><qual:output qual:qualitativeSpecies="v_Fgf8" qual:transitionEffect="assignmentLevel" qual:id="tr_v_Fgf8_out"/></qual:listOfOutputs><qual:listOfFunctionTerms><qual:defaultTerm qual:resultLevel="0"/><qual:functionTerm qual:resultLevel="1"><math xmlns="http://www.w3.org/1998/Math/MathML"><apply><and/><apply><and/><apply><eq/><ci>v_Fgf8</ci><cn type="integer">1</cn></apply><apply><eq/><ci>v_Sp8</ci><cn type="integer">1</cn></apply></apply><apply><not/><apply><eq/><ci>v_Emx2</ci><cn type="integer">1</cn></apply></apply></apply></math></qual:functionTerm></qual:listOfFunctionTerms></qual:transition><qual:transition qual:id="tr_v_Pax6"><qual:listOfInputs><qual:input qual:qualitativeSpecies="v_Coup_fti" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Pax6_in_v_Coup_fti" essential="true"/><qual:input qual:qualitativeSpecies="v_Emx2" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Pax6_in_v_Emx2" essential="true"/><qual:input qual:qualitativeSpecies="v_Sp8" qual:transitionEffect="none" qual:sign="positive" qual:id="tr_v_Pax6_in_v_Sp8" essential="true"/></qual:listOfInputs><qual:listOfOutputs><qual:output qual:qualitativeSpecies="v_Pax6" qual:transitionEffect="assignmentLevel" qual:id="tr_v_Pax6_out"/></qual:listOfOutputs><qual:listOfFunctionTerms><qual:defaultTerm qual:resultLevel="0"/><qual:functionTerm qual:resultLevel="1"><math xmlns="http://www.w3.org/1998/Math/MathML"><apply><and/><apply><eq/><ci>v_Sp8</ci><cn type="integer">1</cn></apply><apply><not/><apply><or/><apply><eq/><ci>v_Emx2</ci><cn type="integer">1</cn></apply><apply><eq/><ci>v_Coup_fti</ci><cn type="integer">1</cn></apply></apply></apply></apply></math></qual:functionTerm></qual:listOfFunctionTerms></qual:transition><qual:transition qual:id="tr_v_Sp8"><qual:listOfInputs><qual:input qual:qualitativeSpecies="v_Emx2" qual:transitionEffect="none" qual:sign="negative" qual:id="tr_v_Sp8_in_v_Emx2" essential="true"/><qual:input qual:qualitativeSpecies="v_Fgf8" qual:transitionEffect="none" qual:sign="positive" qual:id="tr_v_Sp8_in_v_Fgf8" essential="true"/></qual:listOfInputs><qual:listOfOutputs><qual:output qual:qualitativeSpecies="v_Sp8" qual:transitionEffect="assignmentLevel" qual:id="tr_v_Sp8_out"/></qual:listOfOutputs><qual:listOfFunctionTerms><qual:defaultTerm qual:resultLevel="0"/><qual:functionTerm qual:resultLevel="1"><math xmlns="http://www.w3.org/1998/Math/MathML"><apply><and/><apply><eq/><ci>v_Fgf8</ci><cn type="integer">1</cn></apply><apply><not/><apply><eq/><ci>v_Emx2</ci><cn type="integer">1</cn></apply></apply></apply></math></qual:functionTerm></qual:listOfFunctionTerms></qual:transition></qual:listOfTransitions></model></sbml>
`.trim()    // Trim is necessary here because the XML declaration must be at the very start of the file.

const BNET_MODEL = `
targets,factors
v_Coup_fti, (!(v_Fgf8 | v_Sp8) | !(v_Sp8 | v_Fgf8))
v_Emx2, (v_Coup_fti & !((v_Fgf8 | v_Sp8) | v_Pax6))
v_Fgf8, ((v_Fgf8 & v_Sp8) & !v_Emx2)
v_Pax6, (v_Sp8 & !(v_Emx2 | v_Coup_fti))
v_Sp8, (v_Fgf8 & !v_Emx2)
`

const AEON_MODEL = `
v_Sp8 -| v_Emx2
v_Coup_fti -> v_Emx2
v_Pax6 -| v_Emx2
v_Fgf8 -| v_Emx2
v_Emx2 -| v_Sp8
v_Fgf8 -> v_Sp8
v_Sp8 -| v_Coup_fti
v_Fgf8 -| v_Coup_fti
v_Sp8 -> v_Pax6
v_Coup_fti -| v_Pax6
v_Emx2 -| v_Pax6
v_Sp8 -> v_Fgf8
v_Emx2 -| v_Fgf8
v_Fgf8 -> v_Fgf8
$v_Coup_fti: (!(v_Fgf8 | v_Sp8) | !(v_Sp8 | v_Fgf8))
$v_Emx2: (v_Coup_fti & !((v_Fgf8 | v_Sp8) | v_Pax6))
$v_Fgf8: ((v_Fgf8 & v_Sp8) & !v_Emx2)
$v_Pax6: (v_Sp8 & !(v_Emx2 | v_Coup_fti))
$v_Sp8: (v_Fgf8 & !v_Emx2)
`

// Before you first use any of the AEON functions, you have to initialize the WASM module.
await init()

console.log("AEON initialized.")

console.log("Conversion BNET to AEON:", Conversions.bnet_to_aeon(BNET_MODEL).substring(0, 10), "...")
console.log("Conversion SBML to AEON:", Conversions.sbml_to_aeon(SBML_MODEL).substring(0, 10), "...")
console.log("Conversion AEON to BNET:", Conversions.aeon_to_bnet(AEON_MODEL).substring(0, 10), "...")
console.log("Conversion BNET to SBML:", Conversions.aeon_to_sbml(AEON_MODEL).substring(0, 10), "...")

try {
    Conversions.sbml_to_aeon(BNET_MODEL)
} catch (e) {
    console.log(typeof e, e)
}

let model = BooleanNetworkModel.new()
console.log(model.add_variable("x"))
console.log(model.add_variable("y"))
model.free()

console.log(BooleanNetworkModel.from_aeon(AEON_MODEL))

let result = ComputationResult.start(AEON_MODEL)
console.log(result);
console.log(result.get_results())

console.log(check_update_function(`
a -> b
c -|? b
$b: a | f(c)
`))