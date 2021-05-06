/*
 * Copyright IBM Corp. All Rights Reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

/* eslint-disable no-unused-expressions */

const fs = require("fs");
const keyutil = require("jsrsasign").KEYUTIL;
const sinon = require("sinon");
const chai = require("chai");
const chaiAsPromised = require("chai-as-promised");

chai.use(chaiAsPromised);
const { expect } = chai;
chai.should();

const { Wallets } = require("fabric-network");
const { ContractImpl } = require("fabric-network/lib/contract");
const { NetworkImpl } = require("fabric-network/lib/network");
const assetManager = require("../src/AssetManager");
import assetLocksPb from "../protos-js/common/asset_locks_pb";

describe("AssetManager", () => {
    const mspId = "mspId";
    const foreignNetworkId = "foreignNetworkId";
    const userName = "user_name";

    const assetType = "bond";
    const assetID = "A001";
    const fungibleAssetType = "cbdc";
    const numUnits = 1000;
    const recipientECert = fs.readFileSync(`${__dirname}/data/anotherSignCert.pem`).toString();
    let lockerECert;

    let wallet;
    let amc;
    // Initialize wallet with a single user identity
    async function initializeWallet() {
        const privKeyFile = `${__dirname}/data/privKey.pem`;
        const signCertFile = `${__dirname}/data/signCert.pem`;
        const privateKeyStr = fs.readFileSync(privKeyFile).toString();
        const signCert = fs.readFileSync(signCertFile).toString();
        lockerECert = signCert;
        wallet = await Wallets.newInMemoryWallet();
        const userIdentity = {
            credentials: { certificate: signCert, privateKey: privateKeyStr },
            mspId,
            type: "X.509",
        };
        await wallet.put(userName, userIdentity);
        return userIdentity;
    }

    beforeEach(async () => {
        await initializeWallet();
        const network = sinon.createStubInstance(NetworkImpl);
        amc = new ContractImpl(network, "amc", "AssetManager");
    });

    afterEach(() => {
        sinon.restore();
    });

    describe("create HTLC for unique asset", () => {
        let amcStub;

        beforeEach(() => {
            amcStub = sinon.stub(amc, "submitTransaction").resolves(false);
        });

        it("asset lock fails with invalid parameters", async () => {
            let assetLockInvocation = await assetManager.createHTLC(null, assetType, assetID, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createHTLC(amc, "", assetID, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createHTLC(amc, assetType, "", recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createHTLC(amc, assetType, assetID, "", "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
        });

        it("submit asset lock invocation", async () => {
            let assetAgreementStr = assetManager.createAssetExchangeAgreementSerialized(assetType, assetID, recipientECert, "");
            const hashValue = "abcdef123456";
            let expiryTimeSecs = Math.floor(Date.now()/1000) + 300;   // Convert epoch milliseconds to seconds and add 5 minutes
            let lockInfoStr = assetManager.createAssetLockInfoSerialized(hashValue, expiryTimeSecs);
            amcStub.withArgs("LockAsset", assetAgreementStr, lockInfoStr).resolves(true);
            let assetLockInvocation = await assetManager.createHTLC(amc, assetType, assetID, recipientECert, hashValue, expiryTimeSecs);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
            amcStub.withArgs("LockAsset", assetAgreementStr, sinon.match.any).resolves(true);
            assetLockInvocation = await assetManager.createHTLC(amc, assetType, assetID, recipientECert, hashValue, 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
            assetLockInvocation = await assetManager.createHTLC(amc, assetType, assetID, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(20);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
        });
    });

    describe("create HTLC for fungible asset", () => {
        let amcStub;

        beforeEach(() => {
            amcStub = sinon.stub(amc, "submitTransaction").resolves(false);
        });

        it("asset lock fails with invalid parameters", async () => {
            let assetLockInvocation = await assetManager.createFungibleHTLC(null, fungibleAssetType, numUnits, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createFungibleHTLC(amc, "", numUnits, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createFungibleHTLC(amc, fungibleAssetType, -1, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
            assetLockInvocation = await assetManager.createFungibleHTLC(amc, fungibleAssetType, numUnits, "", "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(false);
        });

        it("submit asset lock invocation", async () => {
            let assetAgreementStr = assetManager.createFungibleAssetExchangeAgreementSerialized(fungibleAssetType, numUnits, recipientECert, "");
            const hashValue = "abcdef123456";
            let expiryTimeSecs = Math.floor(Date.now()/1000) + 300;   // Convert epoch milliseconds to seconds and add 5 minutes
            let lockInfoStr = assetManager.createAssetLockInfoSerialized(hashValue, expiryTimeSecs);
            amcStub.withArgs("LockFungibleAsset", assetAgreementStr, lockInfoStr).resolves(true);
            let assetLockInvocation = await assetManager.createFungibleHTLC(amc, fungibleAssetType, numUnits, recipientECert, hashValue, expiryTimeSecs);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
            amcStub.withArgs("LockFungibleAsset", assetAgreementStr, sinon.match.any).resolves(true);
            assetLockInvocation = await assetManager.createFungibleHTLC(amc, fungibleAssetType, numUnits, recipientECert, hashValue, 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(0);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
            assetLockInvocation = await assetManager.createFungibleHTLC(amc, fungibleAssetType, numUnits, recipientECert, "", 0);
            expect(assetLockInvocation).to.be.an('object').that.has.all.keys('preimage', 'result');
            expect(assetLockInvocation.preimage).to.be.a("string");
            expect(assetLockInvocation.preimage.length).to.equal(20);
            expect(assetLockInvocation.result).to.be.a('boolean');
            expect(assetLockInvocation.result).to.equal(true);
        });
    });

    describe("claim unique asset locked in HTLC", () => {
        let amcStub;
        const hashPreimage = "xyz+123-*ty%";

        beforeEach(() => {
            amcStub = sinon.stub(amc, "submitTransaction").resolves(false);
        });

        it("asset claim fails with invalid parameters", async () => {
            let assetClaimInvocation = await assetManager.claimAssetInHTLC(null, assetType, assetID, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimAssetInHTLC(amc, "", assetID, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimAssetInHTLC(amc, assetType, "", lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimAssetInHTLC(amc, assetType, assetID, "", hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimAssetInHTLC(amc, assetType, assetID, lockerECert, "");
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
        });

        it("submit asset claim invocation", async () => {
            let assetAgreementStr = assetManager.createAssetExchangeAgreementSerialized(assetType, assetID, "", lockerECert);
            let claimInfoStr = assetManager.createAssetClaimInfoSerialized(hashPreimage);
            amcStub.withArgs("ClaimAsset", assetAgreementStr, claimInfoStr).resolves(true);
            let assetClaimInvocation = await assetManager.claimAssetInHTLC(amc, assetType, assetID, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(true);
        });
    });

    describe("claim fungible asset locked in HTLC", () => {
        let amcStub;
        const hashPreimage = "xyz+123-*ty%";

        beforeEach(() => {
            amcStub = sinon.stub(amc, "submitTransaction").resolves(false);
        });

        it("asset claim fails with invalid parameters", async () => {
            let assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(null, fungibleAssetType, numUnits, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(amc, "", numUnits, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(amc, fungibleAssetType, -1, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(amc, fungibleAssetType, numUnits, "", hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
            assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(amc, fungibleAssetType, numUnits, lockerECert, "");
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(false);
        });

        it("submit asset claim invocation", async () => {
            let assetAgreementStr = assetManager.createFungibleAssetExchangeAgreementSerialized(fungibleAssetType, numUnits, "", lockerECert);
            let claimInfoStr = assetManager.createAssetClaimInfoSerialized(hashPreimage);
            amcStub.withArgs("ClaimFungibleAsset", assetAgreementStr, claimInfoStr).resolves(true);
            let assetClaimInvocation = await assetManager.claimFungibleAssetInHTLC(amc, fungibleAssetType, numUnits, lockerECert, hashPreimage);
            expect(assetClaimInvocation).to.be.a('boolean');
            expect(assetClaimInvocation).to.equal(true);
        });
    });
});
