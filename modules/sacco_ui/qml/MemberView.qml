import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Member view: my deposits, my loan, SACCO totals (issue #5 checklist).
Item {
    id: root
    property var bridge          // the Main.qml logos bridge
    property string memberAccount: ""
    property string depositAccount: ""
    property var saccoTotals: ({})
    property var myMember: ({})
    property var myDeposit: ({})
    property var myLoans: []
    property string message: ""

    function refresh() {
        if (!bridge || memberAccount === "") return
        try {
            const sacco = JSON.parse(bridge.call("getSacco", []))
            const member = JSON.parse(bridge.call("getMember", [memberAccount]))
            saccoTotals = sacco.data ? (sacco.data.state || sacco.data) : ({})
            myMember = member.data ? (member.data.state || member.data) : ({})
            if (depositAccount !== "") {
                const deposit = JSON.parse(bridge.call("getDeposit", [depositAccount]))
                myDeposit = deposit.data ? (deposit.data.state || deposit.data) : ({})
            } else {
                myDeposit = ({})
            }
            myLoans = []
            if (sacco.status === "indexed_metadata_only" || member.status === "indexed_metadata_only")
                message = "The indexer sees these accounts, but its current API omits shard bytes, so balances and deposit records cannot be decoded yet."
            else
                message = ""
        } catch (e) {
            message = "sacco_core not reachable: " + e
        }
    }
    onMemberAccountChanged: refresh()

    ScrollView {
        id: memberScroll
        anchors.fill: parent
        clip: true

        ColumnLayout {
            x: 16
            y: 16
            width: memberScroll.availableWidth - 32
            spacing: 12

        Label { text: "My account"; font.bold: true; font.pixelSize: 18 }
        RowLayout {
            TextField {
                Layout.fillWidth: true
                placeholderText: "member ledger account ID"
                text: root.memberAccount
                onTextChanged: root.memberAccount = text
            }
            Button { text: "Refresh"; onClicked: root.refresh() }
        }
        Label { text: root.message; color: "firebrick"; visible: root.message !== "" }

        GroupBox {
            title: "SACCO totals (public)"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                Label { text: "Savings deposits" }
                Label { text: root.saccoTotals.total_deposits_savings ?? "—" }
                Label { text: "Share capital" }
                Label { text: root.saccoTotals.total_deposits_share_capital ?? "—" }
                Label { text: "Loans disbursed" }
                Label { text: root.saccoTotals.total_loans_disbursed ?? "—" }
                Label { text: "Loans repaid" }
                Label { text: root.saccoTotals.total_loans_repaid ?? "—" }
                Label { text: "Expenses" }
                Label { text: root.saccoTotals.total_expenses ?? "—" }
            }
        }

        GroupBox {
            title: "My balances"
            Layout.fillWidth: true
            GridLayout {
                columns: 2
                Label { text: "Savings" }
                Label { text: root.myMember.savings_balance ?? "—" }
                Label { text: "Share capital" }
                Label { text: root.myMember.share_capital_balance ?? "—" }
            }
        }

        GroupBox {
            title: "My deposits"
            Layout.fillWidth: true
            ColumnLayout {
                anchors.fill: parent
                RowLayout {
                    Layout.fillWidth: true
                    TextField {
                        Layout.fillWidth: true
                        placeholderText: "deposit ledger account ID"
                        text: root.depositAccount
                        onTextChanged: root.depositAccount = text
                    }
                    Button { text: "Load deposit"; onClicked: root.refresh() }
                }
                Label {
                    Layout.fillWidth: true
                    visible: Object.keys(root.myDeposit).length > 0
                    text: root.myDeposit.amount
                          ? root.myDeposit.amount + " (" + root.myDeposit.deposit_type + ") ref "
                            + root.myDeposit.reference
                            + (root.myDeposit.confirmed_by_member
                               ? "  ✓ confirmed" : "  — awaiting my confirmation")
                          : "No deposit record loaded."
                    wrapMode: Text.Wrap
                }
                Button {
                    text: "Confirm deposit"
                    enabled: Object.keys(root.myDeposit).length > 0
                             && !root.myDeposit.confirmed_by_member
                    onClicked: root.call("confirmDeposit", [root.depositAccount, root.memberAccount])
                }
            }
        }

        GroupBox {
            title: "My loans"
            Layout.fillWidth: true
            ListView {
                width: parent.width; height: Math.min(contentHeight, 120)
                model: root.myLoans
                delegate: Label {
                    text: modelData.principal + " — " + modelData.status +
                          " — repaid " + modelData.repaid_amount
                }
            }
        }

        RowLayout {
            TextField {
                id: loanAmount
                placeholderText: "loan amount"
                validator: IntValidator { bottom: 1 }
            }
            TextField { id: loanPurpose; placeholderText: "purpose" }
            TextField { id: loanAccount; placeholderText: "new loan account" }
            Button {
                text: "Apply for loan"
                onClicked: root.call("applyLoan", [root.memberAccount, loanAccount.text, loanAmount.text, loanPurpose.text])
            }
        }
        }
    }

    function call(method, args) {
        try {
            const response = JSON.parse(bridge.call(method, args))
            if (response.status === "submitted") {
                message = "Deposit confirmed. Transaction " + response.data.tx_hash
                refresh()
            } else {
                message = response.error || "The ledger did not submit the confirmation."
            }
        } catch (e) { message = "Error: " + e }
    }
}
