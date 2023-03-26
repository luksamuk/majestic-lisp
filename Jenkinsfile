podTemplate(containers: [
    containerTemplate(
        name: 'rust',
        image: 'rustlang/rust:nightly',
        command: 'sleep',
        args: '30d'
    )
]) {
    node(POD_LABEL) {
        stage('Geração de Versão') {
            container('rust') {
                stage('Clonar repositório') {
                    git 'https://github.com/luksamuk/majestic-lisp'
                }

		stage('Testes Unitários') {
		    sh 'cargo test'
		}
		
                stage('Compilação') {
                    sh 'cargo build --release'
                }

		stage('Empacotamento') {
		    sh '''
                        cp target/release/majestic-list majestic-lisp
                        MAJESTIC_VERSION=`grep version Cargo.toml | awk '{print $3}' | tr -d '"'`
                        tar -czvf "majestic-lisp-${MAJESTIC_VERSION}.tar.gz" majestic
                        rm majestic-lisp
                    '''
		}
		
		archiveArtifacts artifacts: '*.tar.gz',
		    allowEmptyArchive: false,
		    fingerprint: true,
		    onlyIfSuccessful: true
            }
        }
    }
}
