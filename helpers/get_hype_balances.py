import ijson
import json
from typing import List, Tuple
import os

def extract_user_state(input_file: str) -> ijson.items:
    """Get user_states array iterator"""
    file = open(input_file, 'rb')
    return ijson.items(file, 'exchange.spot_clearinghouse.user_states.item')

def process_user_state(states_iterator) -> List[Tuple[str, str]]:
    """Process states into (address, value) pairs"""
    results = []
    
    for state_array in states_iterator:
        if len(state_array) >= 2 and 'b' in state_array[1]:
            address = state_array[0]
            b_array = state_array[1]['b']
            
            for item in b_array:
                if len(item) >= 2 and item[0] == 150:
                    if isinstance(item[1], dict) and 't' in item[1]:
                        t_value = str(item[1]['t'])
                        results.append((address, t_value))
    
    return results

def sort_pairs(pairs: List[Tuple[str, str]]) -> List[Tuple[str, str]]:
    """Sort pairs by value (largest to smallest)"""
    # Convert to (address, value_str, numeric_value) for sorting
    with_numeric = [(addr, val, float(val)) for addr, val in pairs]
    # Sort by numeric value
    sorted_pairs = sorted(with_numeric, key=lambda x: x[2], reverse=True)
    # Return just the (address, value_str) pairs
    return [(addr, val) for addr, val, _ in sorted_pairs]

def save_to_json(pairs: List[Tuple[str, str]], output_file: str):
    """Save pairs to JSON file in format expected by build_merkle.js"""
    formatted_data = [
        {"address": address, "balance": value}
        for address, value in pairs
    ]
    
    with open(output_file, 'w') as jsonfile:
        json.dump(formatted_data, jsonfile, indent=2)

def main():
    input_file = '../data/state_561930000.json'
    output_file = '../data/balances.json'
    TOP_N = 10000  # Only keep top 10k addresses
    
    try:
        # Chain all operations
        user_state = extract_user_state(input_file)
        address_balance = process_user_state(user_state)
        sorted_balance = sort_pairs(address_balance)
        
        # Take only top 10k addresses
        top_addresses = sorted_balance[:TOP_N]
        save_to_json(top_addresses, output_file)
        
        print(f"Processing complete. Saved top {TOP_N} entries (from {len(sorted_balance)} total) to {output_file}")
        
    except Exception as e:
        print(f"Error: {e}")
        exit(1)

if __name__ == "__main__":
    main() 