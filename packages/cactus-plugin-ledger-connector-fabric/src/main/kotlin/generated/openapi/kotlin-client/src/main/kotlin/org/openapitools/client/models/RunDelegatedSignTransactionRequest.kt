/**
 *
 * Please note:
 * This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * Do not edit this file manually.
 *
 */

@file:Suppress(
    "ArrayInDataClass",
    "EnumEntryName",
    "RemoveRedundantQualifierName",
    "UnusedImport"
)

package org.openapitools.client.models

import org.openapitools.client.models.FabricContractInvocationType
import org.openapitools.client.models.RunTransactionResponseType

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

/**
 * 
 *
 * @param signerCertificate 
 * @param signerMspID 
 * @param channelName 
 * @param contractName 
 * @param invocationType 
 * @param methodName 
 * @param params 
 * @param endorsingPeers An array of endorsing peers (name or url) for the transaction.
 * @param endorsingOrgs An array of endorsing organizations (by mspID or issuer org name on certificate) for the transaction.
 * @param transientData 
 * @param uniqueTransactionData Can be used to uniquely identify and authorize signing request
 * @param responseType 
 */


data class RunDelegatedSignTransactionRequest (

    @Json(name = "signerCertificate")
    val signerCertificate: kotlin.String,

    @Json(name = "signerMspID")
    val signerMspID: kotlin.String,

    @Json(name = "channelName")
    val channelName: kotlin.String,

    @Json(name = "contractName")
    val contractName: kotlin.String,

    @Json(name = "invocationType")
    val invocationType: FabricContractInvocationType,

    @Json(name = "methodName")
    val methodName: kotlin.String,

    @Json(name = "params")
    val params: kotlin.collections.List<kotlin.String> = arrayListOf(),

    /* An array of endorsing peers (name or url) for the transaction. */
    @Json(name = "endorsingPeers")
    val endorsingPeers: kotlin.collections.List<kotlin.String>? = null,

    /* An array of endorsing organizations (by mspID or issuer org name on certificate) for the transaction. */
    @Json(name = "endorsingOrgs")
    val endorsingOrgs: kotlin.collections.List<kotlin.String>? = null,

    @Json(name = "transientData")
    val transientData: kotlin.Any? = null,

    /* Can be used to uniquely identify and authorize signing request */
    @Json(name = "uniqueTransactionData")
    val uniqueTransactionData: kotlin.Any? = null,

    @Json(name = "responseType")
    val responseType: RunTransactionResponseType? = null

)

