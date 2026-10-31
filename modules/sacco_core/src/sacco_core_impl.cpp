#include "sacco_core_impl.h"
#include "sacco_ledger_client_ffi.h"

#include <algorithm>
#include <cctype>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <memory>
#include <stdexcept>

#include "logos_sdk.h"
#include <nlohmann/json.hpp>

#include <sstream>

namespace {

using json = nlohmann::json;

std::string rustResult(char* raw) {
    std::unique_ptr<char, void (*)(char*)> owned(raw, sacco_ledger_client_free);
    if (!owned) return "Rust sacco_ledger client returned no response";
    const json parsed = json::parse(owned.get(), nullptr, false);
    if (!parsed.is_object() || !parsed.value("ok", false)) {
        return parsed.is_object() ? parsed.value("error", "Rust sacco_ledger client error")
                                  : "Rust sacco_ledger client returned invalid JSON";
    }
    return parsed.value("data", json()).dump();
}

std::string instructionVariant(const std::string& name) {
    std::string result;
    bool upper = true;
    for (const char ch : name) {
        if (ch == '_') {
            upper = true;
        } else {
            result.push_back(upper ? static_cast<char>(std::toupper(static_cast<unsigned char>(ch))) : ch);
            upper = false;
        }
    }
    return result;
}

std::string instructionName(const std::string& variant) {
    std::string result;
    for (const char ch : variant) {
        if (std::isupper(static_cast<unsigned char>(ch))) {
            if (!result.empty()) result.push_back('_');
            result.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(ch))));
        } else {
            result.push_back(ch);
        }
    }
    return result;
}

bool isHexAccountId(const std::string& value) {
    return value.size() == 64 && std::all_of(value.begin(), value.end(), [](unsigned char ch) {
        return std::isxdigit(ch) != 0;
    });
}

std::string errorJson(const std::string& message) {
    return json{{"status", "error"}, {"error", message}}.dump();
}

std::string resultJson(const std::string& status, const json& data) {
    return json{{"status", status}, {"data", data}}.dump();
}

std::string jstr(const std::string& value) {
    std::ostringstream out;
    out << '"';
    for (unsigned char c : value) {
        switch (c) {
            case '"': out << "\\\""; break;
            case '\\': out << "\\\\"; break;
            case '\n': out << "\\n"; break;
            case '\r': out << "\\r"; break;
            case '\t': out << "\\t"; break;
            default:
                if (c < 0x20) {
                    out << "\\u00" << "0123456789abcdef"[c >> 4]
                        << "0123456789abcdef"[c & 0x0f];
                } else {
                    out << static_cast<char>(c);
                }
        }
    }
    out << '"';
    return out.str();
}

}  // namespace

void SaccoCoreImpl::openConfiguredLezWallet() {
    const char* home = std::getenv("LEE_WALLET_HOME_DIR");
    if (!home || !*home) home = std::getenv("NSSA_WALLET_HOME_DIR");
    if (!home || !*home) return;

    const std::filesystem::path directory(home);
    const auto config = directory / "wallet_config.json";
    const auto storage = directory / "storage.json";
    if (!std::filesystem::is_regular_file(config) || !std::filesystem::is_regular_file(storage)) return;

    logos::CallError error;
    modules().lez_core.open(config.string(), storage.string(), &error);
}

std::string SaccoCoreImpl::getStatus() {
    // Basecamp calls this during view startup. Keep the health endpoint local and
    // non-blocking: starting the indexer and making synchronous cross-module RPCs
    // here stalls the module process during host initialization.
    json status{{"module", "sacco_core"}, {"api", "sacco_ledger"},
                {"instructions", 18}, {"wallet", isContextReady() ? "configured" : "available"}};
    return status.dump();
}

std::string SaccoCoreImpl::setLedgerContext(const std::string& programId,
                                           const std::string& saccoAccount,
                                           const std::string& feePayerId,
                                           const std::string& signerId) {
    if (!isContextReady()) return errorJson("module context is not ready");
    try {
        programId_ = isHexAccountId(programId) ? programId
                    : modules().lez_core.account_id_from_base58(programId);
        saccoAccount_ = isHexAccountId(saccoAccount) ? saccoAccount
                       : modules().lez_core.account_id_from_base58(saccoAccount);
        feePayerId_ = isHexAccountId(feePayerId) ? feePayerId
                     : modules().lez_core.account_id_from_base58(feePayerId);
        signerId_ = isHexAccountId(signerId) ? signerId
                   : modules().lez_core.account_id_from_base58(signerId);
        if (!isHexAccountId(programId_) || !isHexAccountId(saccoAccount_)
            || !isHexAccountId(feePayerId_) || !isHexAccountId(signerId_)) {
            return errorJson("invalid LEZ account identifier");
        }
        openConfiguredLezWallet();
        logos::CallError walletError;
        const json walletAccounts = modules().lez_core.list_accounts(&walletError);
        if (!walletError.ok() || !walletAccounts.is_array()) {
            return errorJson("LEZ wallet account list is unavailable");
        }
        const auto isOwnedPublic = [&walletAccounts](const std::string& id) {
            return std::any_of(walletAccounts.begin(), walletAccounts.end(), [&id](const json& account) {
                return account.is_object() && account.value("is_public", false)
                    && account.value("account_id", std::string()) == id;
            });
        };
        if (!isOwnedPublic(feePayerId_) || !isOwnedPublic(signerId_)) {
            return errorJson("fee payer and selected signer must be public accounts owned by this wallet");
        }
        return resultJson("ok", {{"program_id", programId_}, {"sacco_account", saccoAccount_},
                                  {"fee_payer", feePayerId_}, {"signer", signerId_}});
    } catch (const std::exception& error) {
        return errorJson(error.what());
    }
}

std::string SaccoCoreImpl::createPublicAccount() {
    if (!isContextReady()) return errorJson("module context is not ready");
    openConfiguredLezWallet();
    const std::string account = modules().lez_core.create_account_public();
    if (account.empty()) return errorJson("LEZ wallet could not create a public account");
    return resultJson("ok", {{"account_id", account}});
}

// ── Setup ────────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::initialize() {
    return dispatchInstruction("Initialize", "{}");
}

std::string SaccoCoreImpl::registerSacco(const std::string& name,
                                         const std::string& registrationNumber,
                                         const std::string& treasury,
                                         const std::string& saccoAccount) {
    const std::string fields = "{\"name\":" + jstr(name) +
                               ",\"registration_number\":" + jstr(registrationNumber) +
                               ",\"treasury\":" + jstr(treasury) + "}";
    json parsed = json::parse(fields);
    parsed["sacco_account"] = saccoAccount;
    const std::string completeFields = parsed.dump();
    return dispatchInstruction("RegisterSacco", completeFields);
}

std::string SaccoCoreImpl::verifySacco() {
    return dispatchInstruction("VerifySacco", "{}");
}

std::string SaccoCoreImpl::addApprover(const std::string& approver) {
    const std::string fields = "{\"approver\":" + jstr(approver) + "}";
    return dispatchInstruction("AddApprover", fields);
}

std::string SaccoCoreImpl::setRequiredApprovals(const std::string& requiredApprovals) {
    const std::string fields = "{\"required_approvals\":" + requiredApprovals + "}";
    return dispatchInstruction("SetRequiredApprovals", fields);
}

std::string SaccoCoreImpl::addSupervisor(const std::string& supervisor) {
    const std::string fields = "{\"supervisor\":" + jstr(supervisor) + "}";
    return dispatchInstruction("AddSupervisor", fields);
}

std::string SaccoCoreImpl::addMember(const std::string& memberAccount,
                                     const std::string& identity) {
    const std::string fields = "{\"identity\":" + jstr(identity) +
                               ",\"member_account\":" + jstr(memberAccount) + "}";
    return dispatchInstruction("AddMember", fields);
}

// ── Deposits ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::recordDeposit(const std::string& memberAccount,
                                         const std::string& depositAccount,
                                         const std::string& amount,
                                         const std::string& depositType,
                                         const std::string& reference) {
    const std::string normalizedType = depositType == "savings" ? "Savings"
        : depositType == "shareCapital" || depositType == "share_capital" ? "ShareCapital" : depositType;
    const std::string fields = "{\"member\":" + jstr(memberAccount) +
                               ",\"amount\":" + amount +
                               ",\"deposit_type\":" + jstr(normalizedType) +
                               ",\"reference\":" + jstr(reference) +
                               ",\"member_account\":" + jstr(memberAccount) +
                               ",\"deposit_account\":" + jstr(depositAccount) + "}";
    return dispatchInstruction("RecordDeposit", fields);
}

std::string SaccoCoreImpl::confirmDeposit(const std::string& depositAccount,
                                          const std::string& memberAccount) {
    const std::string fields = "{\"deposit_account\":" + jstr(depositAccount) +
                               ",\"member_account\":" + jstr(memberAccount) + "}";
    return dispatchInstruction("ConfirmDeposit", fields);
}

// ── Loans ────────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::applyLoan(const std::string& memberAccount,
                                    const std::string& loanAccount,
                                    const std::string& amount,
                                    const std::string& purpose) {
    const std::string fields = "{\"amount\":" + amount +
                               ",\"purpose\":" + jstr(purpose) +
                               ",\"required_approvals\":2,"\
                               "\"member_account\":" + jstr(memberAccount) +
                               ",\"loan_account\":" + jstr(loanAccount) + "}";
    return dispatchInstruction("ApplyLoan", fields);
}

std::string SaccoCoreImpl::approveLoan(const std::string& loanAccount) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) + "}";
    return dispatchInstruction("ApproveLoan", fields);
}

std::string SaccoCoreImpl::rejectLoan(const std::string& loanAccount) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) + "}";
    return dispatchInstruction("RejectLoan", fields);
}

std::string SaccoCoreImpl::disburseLoan(const std::string& memberAccount,
                                        const std::string& loanAccount,
                                        const std::string& evidenceCid,
                                        const std::string& principal) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) +
                               ",\"member_account\":" + jstr(memberAccount) +
                               ",\"evidence_cid\":" + jstr(evidenceCid) +
                               ",\"principal\":" + principal + "}";
    return dispatchInstruction("DisburseLoan", fields);
}

std::string SaccoCoreImpl::recordRepayment(const std::string& memberAccount,
                                           const std::string& loanAccount,
                                           const std::string& amount,
                                           const std::string& reference) {
    const std::string fields = "{\"loan_account\":" + jstr(loanAccount) +
                               ",\"member_account\":" + jstr(memberAccount) +
                               ",\"loan\":" + jstr(loanAccount) +
                               ",\"amount\":" + amount +
                               ",\"reference\":" + jstr(reference) + "}";
    return dispatchInstruction("RecordRepayment", fields);
}

// ── Expenses ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::proposeExpense(const std::string& expenseAccount,
                                          const std::string& amount,
                                          const std::string& purpose,
                                          const std::string& evidenceCid,
                                          const std::string& requiredApprovals) {
    const std::string fields = "{\"amount\":" + amount +
                               ",\"purpose\":" + jstr(purpose) +
                               ",\"evidence_cid\":" + jstr(evidenceCid) +
                               ",\"required_approvals\":" + requiredApprovals +
                               ",\"expense_account\":" + jstr(expenseAccount) + "}";
    return dispatchInstruction("ProposeExpense", fields);
}

std::string SaccoCoreImpl::approveExpense(const std::string& expenseAccount) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) + "}";
    return dispatchInstruction("ApproveExpense", fields);
}

std::string SaccoCoreImpl::rejectExpense(const std::string& expenseAccount) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) + "}";
    return dispatchInstruction("RejectExpense", fields);
}

std::string SaccoCoreImpl::executeExpense(const std::string& expenseAccount,
                                         const std::string& amount,
                                         const std::string& evidenceReference) {
    const std::string fields = "{\"expense_account\":" + jstr(expenseAccount) +
                               ",\"amount\":" + amount +
                               ",\"executed_evidence\":" + jstr(evidenceReference) + "}";
    return dispatchInstruction("ExecuteExpense", fields);
}

// ── Reporting ────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::postPeriodSummary(const std::string& summaryAccount,
                                             const std::string& label,
                                             const std::string& totalsJson) {
    const std::string fields = "{\"summary_account\":" + jstr(summaryAccount) +
                               ",\"label\":" + jstr(label) +
                               ",\"totals\":" + totalsJson + "}";
    return dispatchInstruction("PostPeriodSummary", fields);
}

// ── Read-only queries ────────────────────────────────────────────────────
// Backed by the LEZ indexer once the program id is recorded; until then each
// query reports the account it would read so the views stay testable.

std::string SaccoCoreImpl::getSacco() {
    if (!isContextReady() || saccoAccount_.empty()) return errorJson("SACCO account is not configured");
    return readIndexedAccount("sacco", saccoAccount_);
}

std::string SaccoCoreImpl::getMember(const std::string& memberAccount) {
    if (!isContextReady()) return errorJson("module context is not ready");
    return readIndexedAccount("member", memberAccount);
}

std::string SaccoCoreImpl::getDeposit(const std::string& depositAccount) {
    if (!isContextReady()) return errorJson("module context is not ready");
    return readIndexedAccount("deposit", depositAccount);
}

std::string SaccoCoreImpl::getLoan(const std::string& loanAccount) {
    if (!isContextReady()) return errorJson("module context is not ready");
    return readIndexedAccount("loan", loanAccount);
}

std::string SaccoCoreImpl::getExpense(const std::string& expenseAccount) {
    if (!isContextReady()) return errorJson("module context is not ready");
    return readIndexedAccount("expense", expenseAccount);
}

std::string SaccoCoreImpl::listMyDeposits(const std::string& memberAccount) {
    if (!isContextReady()) return errorJson("module context is not ready");
    const std::string id = isHexAccountId(memberAccount) ? modules().lez_core.account_id_to_base58(memberAccount) : memberAccount;
    const auto history = json::parse(modules().lez_indexer_module.getTransactionsByAccount(id, "0", "100"), nullptr, false);
    if (!history.is_array()) return errorJson("indexer transaction history is unavailable or still syncing");
    return resultJson("indexed", history);
}

std::string SaccoCoreImpl::readIndexedAccount(const std::string& kind,
                                              const std::string& accountId) {
    try {
        // The pinned indexer getAccount endpoint returns account metadata without shard bytes.
        // Read the public account through the wallet's sequencer-backed query instead; its
        // response includes the live shard payloads required by the Rust IDL decoder.
        const std::string id = isHexAccountId(accountId)
            ? accountId
            : modules().lez_core.account_id_from_base58(accountId);
        const json account = json::parse(modules().lez_core.get_account_public(id), nullptr, false);
        if (!account.is_object()) return errorJson("sequencer account is unavailable or still syncing");
        const json* accountData = &account;
        if (account.contains("data") && account["data"].is_object()) {
            accountData = &account["data"];
        }
        const json shards = accountData->value("shards", json::array());
        const auto decodeBytes = [&kind](const json& bytes) -> json {
            if (!bytes.is_array()) return json();
            const json request = {{"kind", kind}, {"bytes", bytes}};
            const json decoded = json::parse(
                rustResult(sacco_ledger_decode_shard_json(request.dump().c_str())), nullptr, false);
            return decoded.is_object() ? decoded : json();
        };
        if (shards.is_array()) {
            for (const auto& shard : shards) {
                if (!shard.is_object() || !shard.contains("data") || !shard["data"].is_string()) continue;
                const std::string hex = shard["data"].get<std::string>();
                if (hex.size() % 2 != 0) continue;
                json bytes = json::array();
                bool valid = true;
                for (std::size_t offset = 0; offset < hex.size(); offset += 2) {
                    try {
                        bytes.push_back(std::stoi(hex.substr(offset, 2), nullptr, 16));
                    } catch (...) {
                        valid = false;
                        break;
                    }
                }
                if (!valid) continue;
                const json decoded = decodeBytes(bytes);
                if (!decoded.is_null()) return resultJson("sequencer", decoded);
            }
        } else if (shards.is_object()) {
            for (auto shard = shards.begin(); shard != shards.end(); ++shard) {
                const json* bytes = &shard.value();
                if (bytes->is_object() && bytes->contains("data")) bytes = &(*bytes)["data"];
                if (bytes->is_array()) {
                    const json decoded = decodeBytes(*bytes);
                    if (!decoded.is_null()) return resultJson("sequencer", decoded);
                } else if (bytes->is_string()) {
                    const std::string hex = bytes->get<std::string>();
                    if (hex.size() % 2 != 0) continue;
                    json byteArray = json::array();
                    bool valid = true;
                    for (std::size_t offset = 0; offset < hex.size(); offset += 2) {
                        try {
                            byteArray.push_back(std::stoi(hex.substr(offset, 2), nullptr, 16));
                        } catch (...) {
                            valid = false;
                            break;
                        }
                    }
                    if (!valid) continue;
                    const json decoded = decodeBytes(byteArray);
                    if (!decoded.is_null()) return resultJson("sequencer", decoded);
                }
            }
        }
        return resultJson("sequencer_account", {{"account_id", id}, {"account", account},
            {"state", nullptr}, {"reason", "sequencer account has no decodable SACCO shard"}});
    } catch (const std::exception& error) {
        return errorJson(error.what());
    }
}

std::string SaccoCoreImpl::listMyLoans(const std::string& memberAccount) {
    return listMyDeposits(memberAccount);
}

std::string SaccoCoreImpl::listPeriodSummaries() {
    if (!isContextReady() || saccoAccount_.empty()) return errorJson("SACCO account is not configured");
    const std::string id = modules().lez_core.account_id_to_base58(saccoAccount_);
    const auto history = json::parse(modules().lez_indexer_module.getTransactionsByAccount(id, "0", "100"), nullptr, false);
    if (!history.is_array()) return errorJson("indexer transaction history is unavailable or still syncing");
    return resultJson("indexed", history);
}

std::string SaccoCoreImpl::getAuditTrail() {
    return listPeriodSummaries();
}

// ── Dispatch ─────────────────────────────────────────────────────────────

std::string SaccoCoreImpl::dispatchInstruction(const std::string& instruction,
                                               const std::string& fieldsJson) {
    if (!isContextReady()) return errorJson("module context is not ready");
    if (programId_.empty() || feePayerId_.empty() || signerId_.empty()) {
        return errorJson("configure program, fee payer, and signer before submitting a transaction");
    }
    const json fields = json::parse(fieldsJson, nullptr, false);
    if (!fields.is_object()) return errorJson("instruction fields must be a JSON object");
    const std::string name = instructionName(instruction);
    const std::string accountMeta = rustResult(sacco_ledger_instruction_accounts_json(name.c_str()));
    const json specs = json::parse(accountMeta, nullptr, false);
    if (!specs.is_array()) return errorJson("Rust IDL has no account table for this instruction");

    json args = fields;
    for (const char* key : {"sacco_account", "member_account", "deposit_account", "loan_account",
                            "expense_account", "summary_account"}) args.erase(key);
    const json instructionJson = args.empty() ? json(instruction) : json{{instruction, args}};
    const std::string encoded = rustResult(sacco_ledger_encode_instruction_json(instructionJson.dump().c_str()));
    const json encodedBytes = json::parse(encoded, nullptr, false);
    if (!encodedBytes.is_array()) return errorJson("Rust IDL rejected the instruction fields: " + encoded);
    std::vector<std::uint8_t> instructionBytes;
    instructionBytes.reserve(encodedBytes.size());
    for (const auto& byte : encodedBytes) {
        if (!byte.is_number_unsigned() && !byte.is_number_integer()) return errorJson("Rust IDL returned invalid instruction bytes");
        const auto value = byte.get<int>();
        if (value < 0 || value > 255) return errorJson("Rust IDL returned an out-of-range instruction byte");
        instructionBytes.push_back(static_cast<std::uint8_t>(value));
    }

    std::vector<std::string> accountIds;
    std::vector<bool> signerRequirements;
    for (const auto& spec : specs) {
        const std::string accountName = spec.value("name", "");
        std::string accountId;
        if (accountName == "sacco") {
            if (instruction == "RegisterSacco") {
                const auto found = fields.find("sacco_account");
                if (found != fields.end() && found->is_string()) {
                    const std::string supplied = found->get<std::string>();
                    accountId = isHexAccountId(supplied) ? supplied
                        : modules().lez_core.account_id_from_base58(supplied);
                }
            } else {
                accountId = saccoAccount_;
            }
        }
        else if (accountName == "admin" || accountName == "approver" || accountName == "recorder"
                 || accountName == "poster" || accountName == "proposer" || accountName == "member_signer") {
            accountId = signerId_;
        } else if (accountName == "config") {
            const std::string programBase58 = modules().lez_core.account_id_to_base58(programId_);
            const json pda = json::parse(rustResult(sacco_ledger_config_pda_json(programBase58.c_str())), nullptr, false);
            if (!pda.is_string()) return errorJson("Rust client could not derive the config PDA");
            accountId = modules().lez_core.account_id_from_base58(pda.get<std::string>());
        } else {
            std::string fieldName = accountName;
            if (accountName == "member") fieldName = "member_account";
            if (accountName == "deposit") fieldName = "deposit_account";
            if (accountName == "loan") fieldName = "loan_account";
            if (accountName == "expense") fieldName = "expense_account";
            if (accountName == "summary") fieldName = "summary_account";
            const auto found = fields.find(fieldName);
            if (found != fields.end() && found->is_string()) {
                const std::string supplied = found->get<std::string>();
                accountId = isHexAccountId(supplied) ? supplied : modules().lez_core.account_id_from_base58(supplied);
            }
        }
        if (!isHexAccountId(accountId)) return errorJson("missing or invalid IDL account input: " + accountName);
        const auto duplicate = std::find(accountIds.begin(), accountIds.end(), accountId);
        if (duplicate == accountIds.end()) {
            accountIds.push_back(accountId);
            signerRequirements.push_back(spec.value("signer", false));
        } else {
            const auto index = static_cast<std::size_t>(std::distance(accountIds.begin(), duplicate));
            // A ledger identity can appear twice in the IDL when an account is
            // both referenced as data and required as a signer (ConfirmDeposit).
            // LEZ accepts one mention per identity, with the strongest signer
            // requirement applied to that single mention.
            signerRequirements[index] = signerRequirements[index] || spec.value("signer", false);
        }
    }
    if (accountIds.empty()) return errorJson("Rust IDL declared no transaction accounts");

    logos::CallError callError;
    const std::string raw = modules().lez_core.send_generic_public_transaction(
        accountIds, signerRequirements, instructionBytes, programId_, feePayerId_, {}, &callError);
    if (!callError.ok()) return errorJson("LEZ wallet submission failed: " + callError.message);
    const json response = json::parse(raw, nullptr, false);
    if (!response.is_object() || !response.value("success", false)
        || response.value("tx_hash", std::string()).empty()) {
        return errorJson(response.is_object() ? response.value("error", "LEZ wallet rejected transaction")
                                               : "LEZ wallet returned invalid transaction JSON");
    }
    operationSubmitted(instruction, response.dump());
    return resultJson("submitted", response);
}
