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


import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

/**
 * Response format from transaction / query execution
 *
 * Values: JSON,UTF8
 */

@JsonClass(generateAdapter = false)
enum class RunTransactionResponseType(val value: kotlin.String) {

    @Json(name = "org.hyperledger.cacti.api.hlfabric.RunTransactionResponseType.JSON")
    JSON("org.hyperledger.cacti.api.hlfabric.RunTransactionResponseType.JSON"),

    @Json(name = "org.hyperledger.cacti.api.hlfabric.RunTransactionResponseType.UTF8")
    UTF8("org.hyperledger.cacti.api.hlfabric.RunTransactionResponseType.UTF8");

    /**
     * Override [toString()] to avoid using the enum variable name as the value, and instead use
     * the actual value defined in the API spec file.
     *
     * This solves a problem when the variable name and its value are different, and ensures that
     * the client sends the correct enum values to the server always.
     */
    override fun toString(): String = value

    companion object {
        /**
         * Converts the provided [data] to a [String] on success, null otherwise.
         */
        fun encode(data: kotlin.Any?): kotlin.String? = if (data is RunTransactionResponseType) "$data" else null

        /**
         * Returns a valid [RunTransactionResponseType] for [data], null otherwise.
         */
        fun decode(data: kotlin.Any?): RunTransactionResponseType? = data?.let {
          val normalizedData = "$it".lowercase()
          values().firstOrNull { value ->
            it == value || normalizedData == "$value".lowercase()
          }
        }
    }
}
