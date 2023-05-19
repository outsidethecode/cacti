/**
 * Hyperledger Cactus Plugin - Connector Fabric
 *
 * Can perform basic tasks on a fabric ledger
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 *
 * Please note:
 * This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * Do not edit this file manually.
 */

@file:Suppress(
    "ArrayInDataClass",
    "EnumEntryName",
    "RemoveRedundantQualifierName",
    "UnusedImport"
)

package org.openapitools.client.models

import org.openapitools.client.models.FabricSigningCredential

import com.squareup.moshi.Json

/**
 * 
 *
 * @param keychain 
 * @param json 
 */

data class GatewayOptionsWallet (

    @Json(name = "keychain")
    val keychain: FabricSigningCredential? = null,

    @Json(name = "json")
    val json: kotlin.String? = null

)
