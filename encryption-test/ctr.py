import binascii
from Crypto.Random import get_random_bytes
from Crypto.Cipher import AES
from Crypto.Util import Counter
import argparse

# length in bytes
KEY_LEN = 16
NONCE_LEN = 16

# counter length in bits
CTR_LEN = 128

def int_of_string(s):
    return int(binascii.hexlify(s), 16)

def byte_xor(ba1, ba2):
    non_xor_len = len(ba1) - len(ba2)
    if non_xor_len < 0:
        non_xor_len = 0

    return ba1[:non_xor_len] + bytes([_a ^ _b for _a, _b in zip(ba1, ba2)])

# corruption is an int (decimal)
def corrupt_ct(ct, corruption):
    # convert corruption int to bytes
    return byte_xor(ct, corruption.to_bytes((corruption.bit_length() // 8) + 1, byteorder="big"))

def encrypt_message(key, plaintext):
    iv = get_random_bytes(NONCE_LEN)
    ctr = Counter.new(CTR_LEN, initial_value=int_of_string(iv))
    aes = AES.new(key, AES.MODE_CTR, counter=ctr)
    return (iv, aes.encrypt(plaintext))

def decrypt_message(key, iv, ciphertext):
    ctr = Counter.new(CTR_LEN, initial_value=int_of_string(iv))
    aes = AES.new(key, AES.MODE_CTR, counter=ctr)
    return aes.decrypt(ciphertext)

# hamming distance is in bits
def hamming_distance(ba1, ba2):
    a1 = bytes_to_int(ba1)
    a2 = bytes_to_int(ba2)

    x = a1 ^ a2
    set_bits = 0

    while (x > 0):
        set_bits += x & 1
        x >>= 1
    
    return set_bits

def bytes_to_int(ba):
    return int.from_bytes(ba, byteorder="big")

def bytes_to_bits(ba):
    return bin(int.from_bytes(ba, byteorder="big"))


if __name__ == "__main__":

    parser = argparse.ArgumentParser(description="Explore malleability of ciphertexts in AES-CTR")
    parser.add_argument('xor_with', type=int, help="integer to xor ciphertext with")
    args = parser.parse_args()

    key = get_random_bytes(KEY_LEN)
    data = b'contagion'

    (iv, ct) = encrypt_message(key, data)
    corrupted_ct = corrupt_ct(ct, args.xor_with) 

    # decrypt original ciphertext
    recover_pt = decrypt_message(key, iv, ct)

    # decrypt corrupted ciphertext
    recover_corr_pt = decrypt_message(key, iv, corrupted_ct)

    assert(hamming_distance(recover_pt, recover_corr_pt) == hamming_distance(ct, corrupted_ct))

