import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Treasurer and committee view: record, propose, approve (issue #5 checklist).
Item {
    id: root
    implicitWidth: 720
    implicitHeight: 520
    property var bridge
    property string message: ""
    property string expenseStatus: ""
    property int expenseApprovalCount: 0
    property int expenseApprovalThreshold: 0

    function refreshExpenseStatus(account) {
        if (!account) return
        try {
            const response = JSON.parse(bridge.call("getExpense", [account]))
            const state = response.data ? (response.data.state || response.data) : ({})
            if (state.status) {
                expenseStatus = state.status
                expenseApprovalCount = (state.approvers || []).length
                expenseApprovalThreshold = state.required_approvals || 0
                message = "Expense " + state.status + " — " + expenseApprovalCount + "/" + expenseApprovalThreshold + " approvals"
            } else {
                message = response.error || "Expense status is not available yet."
            }
        } catch (e) {
            message = "Could not load expense status: " + e
        }
    }

    function call(method, args) {
        try {
            const response = JSON.parse(bridge.call(method, args))
            message = response.status === "submitted"
                    ? "Transaction submitted: " + response.data.tx_hash
                    : (response.error || JSON.stringify(response))
        } catch (e) {
            message = "Error: " + e
        }
    }

    ScrollView {
        id: committeeScroll
        anchors.fill: parent
        clip: true

        ColumnLayout {
            x: 16
            y: 16
            width: committeeScroll.availableWidth - 32
            spacing: 12

            Label { text: "Treasurer and committee"; font.bold: true; font.pixelSize: 18 }
            Label {
                text: root.message
                color: "firebrick"
                visible: root.message !== ""
                Layout.fillWidth: true
                wrapMode: Text.Wrap
            }
            RowLayout {
                Layout.fillWidth: true
                TextField {
                    id: expenseStatusAccount
                    placeholderText: "expense account to inspect"
                    Layout.fillWidth: true
                }
                Button {
                    text: "Refresh expense status"
                    onClicked: root.refreshExpenseStatus(expenseStatusAccount.text)
                }
            }

            GroupBox {
                title: "Record a deposit"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: depMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: depAccount; placeholderText: "new deposit account"; Layout.fillWidth: true }
                        TextField {
                            id: depAmount
                            placeholderText: "amount"
                            validator: IntValidator { bottom: 1 }
                            Layout.fillWidth: true
                        }
                        ComboBox { id: depType; model: ["savings", "shareCapital"]; Layout.fillWidth: true }
                        TextField { id: depRef; placeholderText: "mobile money / bank reference"; Layout.fillWidth: true; Layout.columnSpan: 2 }
                    }
                    Button {
                        text: "Record deposit"
                        onClicked: root.call("recordDeposit",
                            [depMember.text, depAccount.text, depAmount.text, depType.currentText, depRef.text])
                    }
                }
            }

            GroupBox {
                title: "Propose an expense"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField {
                            id: expAmount
                            placeholderText: "amount"
                            validator: IntValidator { bottom: 1 }
                            Layout.fillWidth: true
                        }
                        TextField { id: expAccount; placeholderText: "new expense account"; Layout.fillWidth: true }
                        TextField { id: expPurpose; placeholderText: "purpose"; Layout.fillWidth: true }
                        TextField { id: expCid; placeholderText: "evidence CID (Logos Storage)"; Layout.fillWidth: true }
                        TextField {
                            id: expApprovals
                            placeholderText: "required approvals"
                            text: "2"
                            validator: IntValidator { bottom: 1; top: 32 }
                            Layout.fillWidth: true
                        }
                    }
                    Button {
                        text: "Propose expense"
                        onClicked: {
                            expenseStatusAccount.text = expAccount.text
                            root.call("proposeExpense",
                                [expAccount.text, expAmount.text, expPurpose.text, expCid.text, expApprovals.text])
                        }
                    }
                }
            }

            GroupBox {
                title: "Approvals (multi-signature)"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 3
                        TextField { id: loanAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                        Button { text: "Approve loan"; onClicked: root.call("approveLoan", [loanAccount.text]) }
                        Button { text: "Reject loan"; onClicked: root.call("rejectLoan", [loanAccount.text]) }
                        TextField {
                            id: approvalExpenseAccount
                            placeholderText: "expense account"
                            text: expenseStatusAccount.text
                            Layout.fillWidth: true
                            onTextChanged: expenseStatusAccount.text = text
                        }
                        Button { text: "Approve expense"; onClicked: root.call("approveExpense", [approvalExpenseAccount.text]) }
                        Button { text: "Reject expense"; onClicked: root.call("rejectExpense", [approvalExpenseAccount.text]) }
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        TextField {
                            id: approvalThreshold
                            placeholderText: "new threshold"
                            text: "2"
                            validator: IntValidator { bottom: 1; top: 32 }
                            Layout.fillWidth: true
                        }
                        Button { text: "Set approval threshold"; onClicked: root.call("setRequiredApprovals", [approvalThreshold.text]) }
                    }
                }
            }

            GroupBox {
                title: "Execute approved expense"
                Layout.fillWidth: true
                RowLayout {
                    anchors.fill: parent
                    TextField {
                        id: executionAmount
                        placeholderText: "execution amount"
                        validator: IntValidator { bottom: 1 }
                        Layout.fillWidth: true
                    }
                    TextField { id: executionEvidence; placeholderText: "off-chain payment reference"; Layout.fillWidth: true }
                    Button {
                        text: "Execute"
                        onClicked: root.call("executeExpense", [expAccount.text, executionAmount.text, executionEvidence.text])
                    }
                }
            }

            GroupBox {
                title: "Loan disbursement and repayment"
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: disbAccount; placeholderText: "approved loan account"; Layout.fillWidth: true }
                        TextField { id: disbMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: disbCid; placeholderText: "disbursement evidence CID"; Layout.fillWidth: true }
                        TextField {
                            id: disbPrincipal
                            placeholderText: "principal"
                            validator: IntValidator { bottom: 1 }
                            Layout.fillWidth: true
                        }
                    }
                    Button {
                        text: "Record disbursement"
                        onClicked: root.call("disburseLoan", [disbMember.text, disbAccount.text, disbCid.text, disbPrincipal.text])
                    }
                    GridLayout {
                        Layout.fillWidth: true
                        columns: 2
                        TextField { id: repMember; placeholderText: "member account"; Layout.fillWidth: true }
                        TextField { id: repAccount; placeholderText: "loan account"; Layout.fillWidth: true }
                        TextField {
                            id: repAmount
                            placeholderText: "amount"
                            validator: IntValidator { bottom: 1 }
                            Layout.fillWidth: true
                        }
                        TextField { id: repReference; placeholderText: "repayment reference"; Layout.fillWidth: true }
                    }
                    Button {
                        text: "Record repayment"
                        onClicked: root.call("recordRepayment", [repMember.text, repAccount.text, repAmount.text, repReference.text])
                    }
                }
            }
        }
    }
}
