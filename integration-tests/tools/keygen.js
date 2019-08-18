const Exonum = require('exonum-client');

function main() {
	const keypair = Exonum.keyPair();
	console.log('---');
	console.log(`pub: ${keypair.publicKey}`);
	console.log(`sec: ${keypair.secretKey}`);
}

main();

