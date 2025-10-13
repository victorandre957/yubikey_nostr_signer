#!/bin/bash
# Script para executar o Nostr Bunker

echo "🚀 Nostr Bunker - NIP-46 Remote Signer"
echo ""
echo "Escolha uma opção:"
echo "1) Iniciar servidor bunker"
echo "2) Testar com cliente"
echo "3) Ver logs com tracing"
echo ""
read -p "Opção: " choice

case $choice in
    1)
        echo ""
        echo "Iniciando servidor bunker..."
        cargo run --bin bunker
        ;;
    2)
        echo ""
        echo "Iniciando cliente de teste..."
        cargo run --bin bunker_client
        ;;
    3)
        echo ""
        echo "Iniciando servidor com logs detalhados..."
        RUST_LOG=debug cargo run --bin bunker
        ;;
    *)
        echo "Opção inválida!"
        exit 1
        ;;
esac
