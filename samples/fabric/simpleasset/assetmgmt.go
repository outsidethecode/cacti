/*
 * Copyright IBM Corp. All Rights Reserved.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

package main

import (

    "github.com/golang/protobuf/proto"
    log "github.com/sirupsen/logrus"
    "github.com/hyperledger/fabric-contract-api-go/contractapi"
    "github.com/hyperledger-labs/weaver-dlt-interoperability/common/protos-go/common"
)

// helper functions
// function to return the asset claim information using HTLC
func getClaimInfoProtoBytesHTLC(hashPreimage []byte) ([]byte, error) {
    claimInfoHTLC := &common.AssetClaimHTLC {
        HashPreimageBase64: hashPreimage,
    }
    claimInfoBytes, _ := proto.Marshal(claimInfoHTLC)
    claimInfo := &common.AssetClaim {
        LockMechanism: common.LockMechanism_HTLC,
        ClaimInfo: claimInfoBytes,
    }
    claimInfoProtoBytes, err := proto.Marshal(claimInfo)
    if err != nil {
        return []byte(""), logThenErrorf(err.Error())
    }

    return claimInfoProtoBytes, nil
}

func getLockInfoProtoBytesHTLC(hashBase64 []byte, expiryTimeSecs uint64) ([]byte, error) {
    lockInfoHTLC := &common.AssetLockHTLC {
        HashBase64: hashBase64,
        ExpiryTimeSecs: expiryTimeSecs,
    }
    lockInfoBytes, err := proto.Marshal(lockInfoHTLC)
    if err != nil {
        return []byte(""), logThenErrorf(err.Error())
    }
    lockInfo := &common.AssetLock {
        LockMechanism: common.LockMechanism_HTLC,
        LockInfo: lockInfoBytes,
    }
    lockInfoProtoBytes, err := proto.Marshal(lockInfo)
    if err != nil {
        return []byte(""), logThenErrorf(err.Error())
    }

    return lockInfoProtoBytes, nil
}

// asset specific checks (ideally an asset in a different application might implement checks specific to that asset)
func (s *SmartContract) BondAssetSpecificChecks(ctx contractapi.TransactionContextInterface, assetType, id string, lockInfoSerializedProto64 string) error {

    lockInfo, err := s.amc.ValidateAndExtractLockInfo(lockInfoSerializedProto64)
    if err != nil {
        return err
    }
    lockInfoHTLC := &common.AssetLockHTLC{}
    err = proto.Unmarshal(lockInfo.LockInfo, lockInfoHTLC)
    if err != nil {
        return logThenErrorf("unmarshal error: %+v", err)
    }
    // ReadAsset should check both the existence and ownership of the asset for the locker
    bond, err := s.ReadAsset(ctx, assetType, id, false)
    if err != nil {
        return logThenErrorf("failed reading the bond asset: %+v", err)
    }
    log.Infof("bond: %+v", *bond)
    log.Infof("lockInfoHTLC: %+v", *lockInfoHTLC)

    // Check if asset doesn't mature before locking period
    if uint64(bond.MaturityDate.Unix()) < lockInfoHTLC.ExpiryTimeSecs {
        return logThenErrorf("cannot lock bond asset as it will mature before locking period")
    }

    return nil
}

// Ledger transaction (invocation) functions

func (s *SmartContract) LockAsset(ctx contractapi.TransactionContextInterface, assetExchangeAgreementSerializedProto64 string, lockInfoSerializedProto64 string) (string, error) {

    assetAgreement, err := s.amc.ValidateAndExtractAssetAgreement(assetExchangeAgreementSerializedProto64)
    if err != nil {
        return "", err
    }
    err = s.BondAssetSpecificChecks(ctx, assetAgreement.Type, assetAgreement.Id, lockInfoSerializedProto64)
    if err != nil {
	    return "", logThenErrorf(err.Error())
    }

    contractId, err := s.amc.LockAsset(ctx, assetExchangeAgreementSerializedProto64, lockInfoSerializedProto64)
    if err != nil {
        return "", logThenErrorf(err.Error())
    }

    // write to the ledger the details needed at the time of unlock/claim
    err = s.amc.ContractIdAssetsLookupMap(ctx, assetAgreement.Type, assetAgreement.Id, contractId)
    if err != nil {
        return "", logThenErrorf(err.Error())
    }

    return contractId, nil
}

func (s *SmartContract) LockFungibleAsset(ctx contractapi.TransactionContextInterface, fungibleAssetExchangeAgreementSerializedProto64 string, lockInfoSerializedProto64 string) (string, error) {

    assetAgreement, err := s.amc.ValidateAndExtractFungibleAssetAgreement(fungibleAssetExchangeAgreementSerializedProto64)
    if err != nil {
        return "", err
    }
    lockInfo, err := s.amc.ValidateAndExtractLockInfo(lockInfoSerializedProto64)
    if err != nil {
        return "", err
    }
    lockInfoHTLC := &common.AssetLockHTLC{}
    err = proto.Unmarshal(lockInfo.LockInfo, lockInfoHTLC)
    if err != nil {
	    return "", logThenErrorf("unmarshal error: %+v", err)
    }

    // Check if locker/transaction-creator has enough quantity of token assets to lock
    lockerHasEnoughTokens, err := s.TokenAssetsExist(ctx, assetAgreement.Type, assetAgreement.NumUnits)
    if err != nil {
        return "", logThenErrorf(err.Error())
    }
    if lockerHasEnoughTokens == false {
        return "", logThenErrorf("cannot lock token asset of type %s as there are not enough tokens", assetAgreement.Type)
    }

    contractId, err := s.amc.LockFungibleAsset(ctx, fungibleAssetExchangeAgreementSerializedProto64, lockInfoSerializedProto64)
    if err != nil {
        return "", logThenErrorf(err.Error())
    }

    err = s.DeleteTokenAssets(ctx, assetAgreement.Type, assetAgreement.NumUnits)
    if err != nil {
	// not performing the operation UnlockFungibleAsset and let the TxCreator take care of it
        return contractId, logThenErrorf(err.Error())
    }

    err = s.amc.ContractIdFungibleAssetsLookupMap(ctx, assetAgreement.Type, assetAgreement.NumUnits, contractId)
    if err != nil {
        return "", logThenErrorf(err.Error())
    }

    return contractId, nil
}

// Check whether this asset has been locked by anyone (not just by caller)
func (s *SmartContract) IsAssetLocked(ctx contractapi.TransactionContextInterface, assetAgreementSerializedProto64 string) (bool, error) {
    return s.amc.IsAssetLocked(ctx, assetAgreementSerializedProto64)
}

// Check whether a bond asset has been locked using contractId by anyone (not just by caller)
func (s *SmartContract) IsAssetLockedQueryUsingContractId(ctx contractapi.TransactionContextInterface, contractId string) (bool, error) {
    return s.amc.IsAssetLockedQueryUsingContractId(ctx, contractId)
}

// Check whether a token asset has been locked using contractId by anyone (not just by caller)
func (s *SmartContract) IsFungibleAssetLocked(ctx contractapi.TransactionContextInterface, contractId string) (bool, error) {
    return s.amc.IsFungibleAssetLocked(ctx, contractId)
}

func (s *SmartContract) ClaimAsset(ctx contractapi.TransactionContextInterface, assetAgreementSerializedProto64 string, claimInfoSerializedProto64 string) (bool, error) {
    assetAgreement, err := s.amc.ValidateAndExtractAssetAgreement(assetAgreementSerializedProto64)
    if err != nil {
        return false, err
    }
    claimed, err := s.amc.ClaimAsset(ctx, assetAgreementSerializedProto64, claimInfoSerializedProto64)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if claimed {
        // Change asset ownership to claimant
        recipientECertBase64, err := getECertOfTxCreatorBase64(ctx)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }
        err = s.UpdateOwner(ctx, assetAgreement.Type, assetAgreement.Id, string(recipientECertBase64))
        if err != nil {
            return false, logThenErrorf(err.Error())
        }

	err = s.amc.DeleteAssetLookupMaps(ctx, assetAgreement.Type, assetAgreement.Id)
	if err != nil {
		return false, logThenErrorf("failed to delete bond asset lookup maps: %+v", err)
	}

        return true, nil
    } else {
        return false, logThenErrorf("claim on bond asset type %s with asset id %s failed", assetAgreement.Type, assetAgreement.Id)
    }
}

func (s *SmartContract) ClaimAssetUsingContractId(ctx contractapi.TransactionContextInterface, contractId, claimInfoSerializedProto64 string) (bool, error) {
    claimed, err := s.amc.ClaimAsset(ctx, contractId, claimInfoSerializedProto64)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if claimed {
        // Change asset ownership to claimant
        recipientECertBase64, err := getECertOfTxCreatorBase64(ctx)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }

	// Fetch the contracted bond asset type and id from the ledger
	assetType, assetId, err := s.amc.FetchFromContractIdAssetLookupMap(ctx, contractId)
	if err != nil {
	    return false, logThenErrorf(err.Error())
        }

        err = s.UpdateOwner(ctx, assetType, assetId, recipientECertBase64)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }
	// delete the lookup maps
	err = s.amc.DeleteAssetLookupMapsUsingContractId(ctx, assetType, assetId, contractId)

        return true, nil
    } else {
        return false, logThenErrorf("claim on bond asset using contractId %s failed", contractId)
    }
}

func (s *SmartContract) ClaimFungibleAsset(ctx contractapi.TransactionContextInterface, contractId, claimInfoSerializedProto64 string) (bool, error) {
    claimed, err := s.amc.ClaimFungibleAsset(ctx, contractId, claimInfoSerializedProto64)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if claimed {
        // Add the claimed tokens into the wallet of the claimant
        recipientECertBase64, err := getECertOfTxCreatorBase64(ctx)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }

	// Fetch the contracted token asset type and numUnits from the ledger
	assetType, numUnits, err := s.amc.FetchFromContractIdFungibleAssetLookupMap(ctx, contractId)
	if err != nil {
	    return false, logThenErrorf(err.Error())
        }

        err = s.IssueTokenAssets(ctx, assetType, numUnits, recipientECertBase64)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }
	err = s.amc.DeleteFungibleAssetLookupMap(ctx, contractId)
	if err != nil {
            return false, logThenErrorf(err.Error())
	}
        return true, nil
    } else {
        return false, logThenErrorf("claim on token asset using contractId %s failed", contractId)
    }
}

func (s *SmartContract) UnlockAsset(ctx contractapi.TransactionContextInterface, assetAgreementSerializedProto64 string) (bool, error) {
    assetAgreement, err := s.amc.ValidateAndExtractAssetAgreement(assetAgreementSerializedProto64)
    if err != nil {
        return false, err
    }

    unlocked, err := s.amc.UnlockAsset(ctx, assetAgreementSerializedProto64)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if unlocked {
	err = s.amc.DeleteAssetLookupMaps(ctx, assetAgreement.Type, assetAgreement.Id)
	if err != nil {
            return false, logThenErrorf("failed to delete bond asset lookup maps: %+v", err)
        }
    } else {
        return false, logThenErrorf("unlock on bond asset type %s with asset id %s failed", assetAgreement.Type, assetAgreement.Id)
    }

    return true, nil
}

func (s *SmartContract) UnlockBondAssetUsingContractId(ctx contractapi.TransactionContextInterface, contractId string) (bool, error) {
    unlocked, err := s.amc.UnlockAssetUsingContractId(ctx, contractId)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if unlocked {
        // delete the lookup maps
        err := s.amc.DeleteAssetLookupMapsOnlyUsingContractId(ctx, contractId)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }
        return true, nil
    } else {
        return false, logThenErrorf("unlock on bond asset using contractId %s failed", contractId)
    }
}

func (s *SmartContract) UnlockFungibleAsset(ctx contractapi.TransactionContextInterface, contractId string) (bool, error) {
    unlocked, err := s.amc.UnlockFungibleAsset(ctx, contractId)
    if err != nil {
        return false, logThenErrorf(err.Error())
    }
    if unlocked {
        // Add the unlocked tokens into the wallet of the locker
        lockerECertBase64, err := getECertOfTxCreatorBase64(ctx)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }

	// Fetch the contracted token asset type and numUnits from the ledger
	assetType, numUnits, err := s.amc.FetchFromContractIdFungibleAssetLookupMap(ctx, contractId)
	if err != nil {
	    return false, logThenErrorf(err.Error())
        }

        err = s.IssueTokenAssets(ctx, assetType, numUnits, lockerECertBase64)
        if err != nil {
            return false, logThenErrorf(err.Error())
        }
	err = s.amc.DeleteFungibleAssetLookupMap(ctx, contractId)
	if err != nil {
            return false, logThenErrorf(err.Error())
        }
        return true, nil
    } else {
        return false, logThenErrorf("unlock on token asset using contractId %s failed", contractId)
    }
}
