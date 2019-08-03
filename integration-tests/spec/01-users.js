const Promise = require('bluebird');
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Users = require('../helpers/users');

describe('users', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	beforeAll((done) => {
		bootstrap.load().then(done).catch(done.fail);
	});

	afterAll((done) => {
		trans.clear_users();
		bootstrap.unload().then(done).catch(done.fail);
	});

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	const sandra_user_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	trans.add_user('root', config.bootstrap_user.pub, config.bootstrap_user.sec);
	trans.add_user('jerry', jerry_pubkey, jerry_seckey);
	trans.add_user('sandra', sandra_pubkey, sandra_seckey);

	it('can be created', async () => {
		// create jerry
		let jerry = await Users.get({id: jerry_user_id});
		expect(jerry).toBe(null);
		var txid = await trans.send_as('root', tx.user.TxCreate, {
			id: jerry_user_id,
			pubkey: jerry_pubkey,
			roles: ['User'],
			email: jerry_email,
			name: 'jerry anderson',
			meta: '{}',
			created: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		jerry = await Users.get({id: jerry_user_id});
		expect(jerry.id).toBe(jerry_user_id);

		// create sandra
		let sandra = await Users.get({id: sandra_user_id});
		expect(sandra).toBe(null);
		var txid = await trans.send_as('root', tx.user.TxCreate, {
			id: sandra_user_id,
			pubkey: sandra_pubkey,
			roles: ['User'],
			email: sandra_email,
			name: 'sandra sanderton',
			meta: '{}',
			created: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		sandra = await Users.get({id: sandra_user_id});
		expect(sandra.id).toBe(sandra_user_id);

		// needs a valid email
		const slappy_user_id = uuid();
		let slappy = await Users.get({id: slappy_user_id});
		const {publicKey: slappy_pubkey, secretKey: slappy_seckey} = Exonum.keyPair();
		expect(slappy).toBe(null);
		var txid = await trans.send_as('root', tx.user.TxCreate, {
			id: slappy_user_id,
			pubkey: slappy_pubkey,
			roles: ['User'],
			email: 'HIGH ENERGY ALPHA MALE',
			name: 'slappy',
			meta: '',
			created: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(false);
		expect(status.description.match(/invalid email/i)).toBeTruthy();
	});

	it('can update themselves', async () => {
		var txid = await trans.send_as('jerry', tx.user.TxUpdate, {
			id: jerry_user_id,
			email: jerry_email_new,
			name: 'jerry *THE JERJER* anderson',
			meta: '{"friends":99}',
			updated: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).toBe(jerry_email_new);
	})

	it('stops users from editing other users', async () => {
		// sandra cannot update jerjer
		var data2 = {
			id: jerry_user_id,
			email: 'jerry.sux@jerry.is.NOT.cool.net',
			name: 'jerry *THE JERKJERK* JERKSTON',
			meta: '{"friends":0}',
			updated: new Date().toISOString(),
		};
		var txid = await trans.send_as('sandra', tx.user.TxUpdate, data2);
		var status = await trans.wait(txid);
		expect(status.success).toBe(false);
		expect(status.committed).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).toBe(jerry_email_new);
		expect(jerry.email).not.toBe(data2.email);
	});

	it('can have their permissions changed by admin', async () => {
		var txid = await trans.send_as('root', tx.user.TxSetRoles, {
			id: sandra_user_id,
			roles: ['IdentityAdmin'],
			memo: 'great job, sandra. you\'ve earned this',
			updated: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
	});

	it('lets users with correct permissions edit other users', async () => {
		// sandra CAN NOW abuse her power and update jerjer the jerkjerk
		var txid = await trans.send_as('sandra', tx.user.TxUpdate, {
			id: jerry_user_id,
			email: 'jerry.sux@jerry.is.NOT.cool.net',
			name: 'jerry *THE JERKJERK* JERKSTON',
			meta: '{"friends":0}',
			updated: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).not.toBe(jerry_email_new);
	});

	it('can have their pubkey changed by an admin', async () => {
		const {publicKey: sandra_pubkey2, secretKey: sandra_seckey2} = Exonum.keyPair();
		var txid = await trans.send_as('root', tx.user.TxSetPubkey, {
			id: sandra_user_id,
			pubkey: sandra_pubkey2,
			memo: 'sandra lost her key. again.',
			updated: new Date().toISOString(),
		});
		trans.add_user('sandra2', sandra_pubkey2, sandra_seckey2);
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
	});

	it('cannot update with an old pubkey', async () => {
		var txid = await trans.send_as('sandra', tx.user.TxUpdate, {
			id: sandra_user_id,
			email: 'sandra@is.kewl.net',
			updated: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(false);
		var sandra = await Users.get({id: sandra_user_id});
		expect(sandra.email).toBe(sandra_email);
	});

	it('can self-delete', async () => {
		var user = await Users.get({id: jerry_user_id});
		expect(user.id).toBe(jerry_user_id);
		var txid = await trans.send_as('jerry', tx.user.TxDelete, {
			id: jerry_user_id,
			memo: `just gettin deleted, huh? thats cool...`,
			deleted: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		user = await Users.get({id: jerry_user_id});
		expect(user).toBe(null);

		var user = await Users.get({email: sandra_email});
		expect(user.id).toBe(sandra_user_id);
		var txid = await trans.send_as('sandra2', tx.user.TxDelete, {
			id: sandra_user_id,
			memo: `i hate this stupid system`,
			deleted: new Date().toISOString(),
		});
		var status = await trans.wait(txid);
		expect(status.success).toBe(true);
		user = await Users.get({id: sandra_user_id});
		expect(user).toBe(null);
	});
});

