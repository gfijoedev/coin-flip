import * as dotenv from 'dotenv';
dotenv.config();
import { Account } from '@near-js/accounts';
import { JsonRpcProvider } from '@near-js/providers';
import { KeyPairSigner } from '@near-js/signers';
import { KeyPair } from '@near-js/crypto';
import { NEAR } from '@near-js/tokens';
import { parseSeedPhrase } from 'near-seed-phrase';

const wait = async (s) => await new Promise((r) => setTimeout(r, s));
const provider = new JsonRpcProvider({
    url: 'https://test.rpc.fastnear.com',
});

// setup account
const { publicKey, secretKey } = parseSeedPhrase(process.env.NEAR_SEED_PHRASE);
const keyPair = KeyPair.fromString(secretKey);
let signer = new KeyPairSigner(keyPair);

async function getBalance(accountId) {
    const account = new Account(
        accountId || process.env.NEAR_ACCOUNT_ID,
        provider,
        signer,
    );
    const amount = await account.getBalance(NEAR);
    console.log(NEAR.toDecimal(amount));
}

async function main() {
    await getBalance();

    const faucetUrl = 'https://helper.nearprotocol.com/account';
    const newAccountId = Date.now() + '-gfijoe.testnet';
    const newAccountPublicKey = publicKey.toString();

    const res = await fetch(faucetUrl, {
        method: 'POST',
        body: JSON.stringify({
            newAccountId,
            newAccountPublicKey,
        }),
    });

    console.log(res.status);
    console.log((await res.json()).status.SuccessValue === '');

    await wait(1000);

    const account = new Account(newAccountId, provider, signer);
    account.deleteAccount('gfijoe.testnet');

    // wait random 5-10 mins
    const waitS = 300000 + Math.floor(Math.random() * 5) + 60000;
    console.log('waiting mins', waitS / 60000);
    await wait(waitS);

    main();
}

main();
