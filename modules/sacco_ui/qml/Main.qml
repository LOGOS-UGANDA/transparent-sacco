import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    width: 720
    height: 520

    property string statusText: "Loading sacco_core…"
    property string lastResult: ""

    function refreshStatus() {
        try {
            statusText = logos.callModule("sacco_core", "getStatus", [])
        } catch (e) {
            statusText = "sacco_core not loaded — install the core module first"
        }
    }

    function callCore(method, args) {
        try {
            lastResult = bridge.call(method, args)
        } catch (e) {
            lastResult = "Error: " + e
        }
    }

    Component.onCompleted: refreshStatus()

    // Bridge the scaffold's callModule API to the view-level `call(method, args)`.
    property var bridge: QtObject {
        function call(method, args) {
            const raw = logos.callModule("sacco_core", method, args)
            if (typeof raw !== "string") {
                root.lastResult = JSON.stringify(raw)
                return root.lastResult
            }
            try {
                const decoded = JSON.parse(raw)
                root.lastResult = typeof decoded === "string" ? decoded : JSON.stringify(decoded)
            } catch (e) {
                root.lastResult = raw
            }
            return root.lastResult
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        Text {
            text: "Transparent SACCO"
            font.pixelSize: 24
            font.bold: true
            Layout.fillWidth: true
        }

        Text {
            text: "Off-chain SACCO activity. On-chain records and approvals."
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
            opacity: 0.8
        }

        Text {
            text: "Core status: " + statusText
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        GroupBox {
            title: "Local LEZ transaction context"
            Layout.fillWidth: true
            Layout.preferredHeight: 216
            GridLayout {
                anchors.fill: parent
                columns: 2
                TextField { id: programId; placeholderText: "sacco_ledger program account ID"; Layout.fillWidth: true }
                TextField { id: saccoId; placeholderText: "SACCO account ID"; Layout.fillWidth: true }
                TextField { id: feePayerId; placeholderText: "funded fee payer account ID"; Layout.fillWidth: true }
                TextField { id: signerId; placeholderText: "selected signer account ID"; Layout.fillWidth: true }
                Button {
                    text: "Set transaction context"
                    onClicked: callCore("setLedgerContext", [programId.text, saccoId.text, feePayerId.text, signerId.text])
                }
                Button {
                    text: "Create public account"
                    onClicked: callCore("createPublicAccount", [])
                }
                Button { text: "Initialize ledger"; onClicked: callCore("initialize", []) }
            }
        }

        TabBar {
            id: roleTabs
            Layout.fillWidth: true
            TabButton { text: "Member" }
            TabButton { text: "Committee" }
            TabButton { text: "Auditor" }
        }

        StackLayout {
            currentIndex: roleTabs.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 180
            Loader {
                active: roleTabs.currentIndex === 0
                source: "MemberView.qml"
                Layout.fillWidth: true
                Layout.fillHeight: true
                onLoaded: item.bridge = root.bridge
            }
            Loader {
                active: roleTabs.currentIndex === 1
                source: "CommitteeView.qml"
                Layout.fillWidth: true
                Layout.fillHeight: true
                onLoaded: item.bridge = root.bridge
            }
            Loader {
                active: roleTabs.currentIndex === 2
                source: "AuditorView.qml"
                Layout.fillWidth: true
                Layout.fillHeight: true
                onLoaded: item.bridge = root.bridge
            }
        }

        GroupBox {
            title: "Last core response"
            Layout.fillWidth: true
            Layout.preferredHeight: 70
            TextArea {
                anchors.fill: parent
                readOnly: true
                wrapMode: TextEdit.Wrap
                text: lastResult.length ? lastResult : "No call yet."
                selectByMouse: true
            }
        }
    }
}
