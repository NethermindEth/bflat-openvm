/**
 * @file
 * @brief Host tests for the OpenVM accelerator glue.
 *
 * The OpenVM custom instructions cannot run here, so this links the real
 * sponge/padding code against SOFTWARE stand-ins for the two primitives
 * OpenVM accelerates. That is deliberate: the permutation and the compression
 * function are OpenVM's to get right, while the padding, the block loop and
 * the boundary cases are ours - and they are where the bugs live.
 *
 * Copyright (C) 2026 Demerzel Solutions Limited (Nethermind)
 */
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

/* ---- software stand-in: Keccak-f[1600] --------------------------------- */
static const uint64_t RC[24] = {
    0x0000000000000001ULL,0x0000000000008082ULL,0x800000000000808aULL,0x8000000080008000ULL,
    0x000000000000808bULL,0x0000000080000001ULL,0x8000000080008081ULL,0x8000000000008009ULL,
    0x000000000000008aULL,0x0000000000000088ULL,0x0000000080008009ULL,0x000000008000000aULL,
    0x000000008000808bULL,0x800000000000008bULL,0x8000000000008089ULL,0x8000000000008003ULL,
    0x8000000000008002ULL,0x8000000000000080ULL,0x000000000000800aULL,0x800000008000000aULL,
    0x8000000080008081ULL,0x8000000000008080ULL,0x0000000080000001ULL,0x8000000080008008ULL };
static const int RHO[24] = {1,3,6,10,15,21,28,36,45,55,2,14,27,41,56,8,25,43,62,18,39,61,20,44};
static const int PI[24] = {10,7,11,17,18,3,5,16,8,21,24,4,15,23,19,13,12,2,20,14,22,9,6,1};
#define ROL(x,s) (((x) << (s)) | ((x) >> (64 - (s))))

void zkvm_keccakf(uint8_t *state)
{
    uint64_t a[25];
    memcpy(a, state, 200);
    for (int r = 0; r < 24; r++) {
        uint64_t b[5], t;
        for (int i = 0; i < 5; i++) b[i] = a[i] ^ a[i+5] ^ a[i+10] ^ a[i+15] ^ a[i+20];
        for (int i = 0; i < 5; i++) {
            t = b[(i+4)%5] ^ ROL(b[(i+1)%5], 1);
            for (int j = 0; j < 25; j += 5) a[j+i] ^= t;
        }
        t = a[1];
        for (int i = 0; i < 24; i++) { int j = PI[i]; uint64_t tmp = a[j]; a[j] = ROL(t, RHO[i]); t = tmp; }
        for (int j = 0; j < 25; j += 5) {
            uint64_t c[5];
            for (int i = 0; i < 5; i++) c[i] = a[j+i];
            for (int i = 0; i < 5; i++) a[j+i] = c[i] ^ ((~c[(i+1)%5]) & c[(i+2)%5]);
        }
        a[0] ^= RC[r];
    }
    memcpy(state, a, 200);
}

void zkvm_keccak_xorin(uint8_t *state, const uint8_t *input, size_t len)
{
    for (size_t i = 0; i < len; i++) state[i] ^= input[i];
}

/* ---- software stand-in: SHA-256 compression ---------------------------- */
static const uint32_t K[64] = {
0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2 };
#define ROR(x,n) (((x) >> (n)) | ((x) << (32-(n))))

void zkvm_sha256_compress(const uint8_t *state, const uint8_t *input, uint8_t *output)
{
    uint32_t h[8], w[64];
    for (int i = 0; i < 8; i++)
        h[i] = ((uint32_t)state[4*i]<<24)|((uint32_t)state[4*i+1]<<16)|((uint32_t)state[4*i+2]<<8)|state[4*i+3];
    for (int i = 0; i < 16; i++)
        w[i] = ((uint32_t)input[4*i]<<24)|((uint32_t)input[4*i+1]<<16)|((uint32_t)input[4*i+2]<<8)|input[4*i+3];
    for (int i = 16; i < 64; i++) {
        uint32_t s0 = ROR(w[i-15],7) ^ ROR(w[i-15],18) ^ (w[i-15] >> 3);
        uint32_t s1 = ROR(w[i-2],17) ^ ROR(w[i-2],19) ^ (w[i-2] >> 10);
        w[i] = w[i-16] + s0 + w[i-7] + s1;
    }
    uint32_t a=h[0],b=h[1],c=h[2],d=h[3],e=h[4],f=h[5],g=h[6],hh=h[7];
    for (int i = 0; i < 64; i++) {
        uint32_t S1 = ROR(e,6)^ROR(e,11)^ROR(e,25);
        uint32_t ch = (e & f) ^ ((~e) & g);
        uint32_t t1 = hh + S1 + ch + K[i] + w[i];
        uint32_t S0 = ROR(a,2)^ROR(a,13)^ROR(a,22);
        uint32_t mj = (a & b) ^ (a & c) ^ (b & c);
        uint32_t t2 = S0 + mj;
        hh=g; g=f; f=e; e=d+t1; d=c; c=b; b=a; a=t1+t2;
    }
    h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d; h[4]+=e; h[5]+=f; h[6]+=g; h[7]+=hh;
    for (int i = 0; i < 8; i++) {
        output[4*i]   = (uint8_t)(h[i] >> 24); output[4*i+1] = (uint8_t)(h[i] >> 16);
        output[4*i+2] = (uint8_t)(h[i] >> 8);  output[4*i+3] = (uint8_t)h[i];
    }
}

/* ---- the code under test ----------------------------------------------- */
#include "../src/accelerators/zkvm_accelerators.h"

static int failures = 0;

static void check(const char *name, const uint8_t *got, const char *want_hex)
{
    char hex[65];
    for (int i = 0; i < 32; i++) sprintf(hex + 2*i, "%02x", got[i]);
    hex[64] = 0;
    if (strcmp(hex, want_hex) != 0) {
        printf("FAIL %s\n  got  %s\n  want %s\n", name, hex, want_hex);
        failures++;
    } else {
        printf("ok   %s\n", name);
    }
}

int main(void)
{
    zkvm_keccak256_hash k;
    zkvm_sha256_hash s;
    uint8_t buf[1000];

    zkvm_keccak256((const uint8_t *)"", 0, &k);
    check("keccak256(\"\")", k.bytes, "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");

    zkvm_keccak256((const uint8_t *)"abc", 3, &k);
    check("keccak256(\"abc\")", k.bytes, "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45");

    /* rate - 1: the padding start and the 0x80 land in the same byte */
    for (int i = 0; i < 135; i++) buf[i] = (uint8_t)i;
    zkvm_keccak256(buf, 135, &k);
    check("keccak256(135B, padding collision)", k.bytes,
          "cbdfd9dee5faad3818d6b06f95a219fd290b0e1706f6a82e5a595b9ce9faca62");

    /* exactly one rate-sized block: forces a whole extra padding block */
    for (int i = 0; i < 136; i++) buf[i] = (uint8_t)i;
    zkvm_keccak256(buf, 136, &k);
    check("keccak256(136B, exact block)", k.bytes,
          "7ce759f1ab7f9ce437719970c26b0a66ff11fe3e38e17df89cf5d29c7d7f807e");

    /* two full blocks: exercises the absorb loop */
    for (int i = 0; i < 272; i++) buf[i] = (uint8_t)(i & 0xff);
    zkvm_keccak256(buf, 272, &k);
    check("keccak256(272B, two blocks)", k.bytes,
          "fdf2ec49e749960d3c8521a0219af8d03e30e2b3bf19bd16150ee0eaf133d66e");

    zkvm_sha256((const uint8_t *)"", 0, &s);
    check("sha256(\"\")", s.bytes, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

    zkvm_sha256((const uint8_t *)"abc", 3, &s);
    check("sha256(\"abc\")", s.bytes, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");

    /* 56 bytes: the length field no longer fits, forcing the extra block */
    zkvm_sha256((const uint8_t *)"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", 56, &s);
    check("sha256(56B, extra block)", s.bytes, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");

    /* exactly one block */
    for (int i = 0; i < 64; i++) buf[i] = 'a';
    zkvm_sha256(buf, 64, &s);
    check("sha256(64B)", s.bytes, "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb");

    printf("%s: %d failure(s)\n", failures ? "FAILED" : "PASSED", failures);
    return failures != 0;
}
