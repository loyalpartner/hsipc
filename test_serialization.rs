// Test serialization/deserialization
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AddRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AddResponse {
    pub result: i32,
}

fn main() {
    // Test if () can be serialized/deserialized
    let unit = ();
    let serialized = bincode::serialize(&unit).unwrap();
    println!("Serialized (): {:?}", serialized);
    
    let deserialized: () = bincode::deserialize(&serialized).unwrap();
    println!("Deserialized (): {:?}", deserialized);
    
    // Test with actual response
    let response = AddResponse { result: 15 };
    let serialized_response = bincode::serialize(&response).unwrap();
    println!("Serialized response: {:?}", serialized_response);
    
    let deserialized_response: AddResponse = bincode::deserialize(&serialized_response).unwrap();
    println!("Deserialized response: {:?}", deserialized_response);
    
    // Test deserializing response as ()
    let deserialized_unit: () = bincode::deserialize(&serialized_response).unwrap();
    println!("Deserialized response as unit: {:?}", deserialized_unit);
}