---
{
  "title": "Bitbucket Pipelines",
  "slug": "bitbucket-pipelines",
  "description": "Déployez votre documentation DocAnvil sur AWS S3 avec Bitbucket Pipelines"
}
---
# Déploiement avec Bitbucket Pipelines

Ce guide explique comment configurer un pipeline Bitbucket qui compile votre documentation DocAnvil et déploie le résultat statique sur un bucket AWS S3 — à chaque push sur `master`.

## Prérequis

Vous aurez besoin de :

- Un dépôt Bitbucket contenant votre projet DocAnvil
- Un compte AWS avec un bucket S3 configuré pour l'hébergement de site statique (ou CloudFront devant un bucket privé)
- Un utilisateur ou rôle IAM avec les permissions d'écriture sur ce bucket (et optionnellement de créer des invalidations CloudFront)

La politique IAM minimale pour un déploiement S3 uniquement :

```json
{
  "Effect": "Allow",
  "Action": ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket"],
  "Resource": [
    "arn:aws:s3:::nom-de-votre-bucket",
    "arn:aws:s3:::nom-de-votre-bucket/*"
  ]
}
```

Ajoutez `cloudfront:CreateInvalidation` à la politique si vous utilisez CloudFront.

## Variables du dépôt

Ajoutez les variables suivantes dans **Paramètres du dépôt → Variables du dépôt** (ou dans un environnement de déploiement nommé) :

| Variable | Requise | Description |
|---|---|---|
| `AWS_ACCESS_KEY_ID` | ✅ | Identifiant de clé d'accès IAM |
| `AWS_SECRET_ACCESS_KEY` | ✅ | Clé secrète IAM — marquez-la comme **Sécurisée** |
| `AWS_DEFAULT_REGION` | ✅ | Région AWS, ex. `us-east-1` |
| `S3_BUCKET` | ✅ | Nom du bucket S3, ex. `docs.exemple.com` |
| `CLOUDFRONT_DISTRIBUTION_ID` | Optionnelle | Identifiant de distribution CloudFront pour l'invalidation du cache |

## Configuration du pipeline

Créez `bitbucket-pipelines.yml` à la racine de votre dépôt :

```yaml
image: atlassian/default-image:4

pipelines:
  branches:
    master:
      - step:
          name: Compiler et déployer la documentation
          deployment: production
          script:
            # Télécharger le dernier binaire DocAnvil
            - LATEST=$(curl -sSf https://api.github.com/repos/docanvil/docanvil/releases/latest | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
            - VERSION=${LATEST#v}
            - curl -sSfL "https://github.com/docanvil/docanvil/releases/download/${LATEST}/docanvil-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar -xz -C /usr/local/bin

            # Compiler la documentation
            - docanvil build --strict --clean --path ./docs

            # Synchroniser le résultat statique vers S3
            - aws s3 sync docs/dist/ s3://$S3_BUCKET --delete

            # Invalider le cache CloudFront (décommenter si vous utilisez CloudFront)
            # - aws cloudfront create-invalidation --distribution-id $CLOUDFRONT_DISTRIBUTION_ID --paths "/*"
```

Quelques points à noter :

- **`atlassian/default-image:4`** inclut l'AWS CLI nativement — aucune étape d'installation supplémentaire n'est nécessaire.
- **`deployment: production`** rattache l'étape à votre environnement de déploiement Bitbucket. Les variables définies sur cet environnement ont priorité sur les variables au niveau du dépôt.
- **`--strict`** fait échouer le build en cas d'avertissements — les liens wiki cassés, les images manquantes et autres problèmes stoppent le déploiement avant qu'il n'atteigne S3.
- **`--delete`** dans la synchronisation S3 supprime les objets qui n'existent plus dans le résultat compilé, ce qui maintient le bucket propre.

## Adapter à la structure de votre projet

L'exemple ci-dessus suppose que votre projet DocAnvil se trouve dans un sous-répertoire `docs/` de votre dépôt. Adaptez le chemin `--path` et la source de synchronisation en fonction de votre configuration :

```yaml
# Projet DocAnvil à la racine du dépôt
- docanvil build --strict --clean
- aws s3 sync dist/ s3://$S3_BUCKET --delete

# Projet DocAnvil dans un sous-répertoire
- docanvil build --strict --clean --path ./ma-doc
- aws s3 sync ma-doc/dist/ s3://$S3_BUCKET --delete
```

## Configuration du bucket S3

Pour l'hébergement de site statique public, activez **Hébergement de site web statique** dans la console S3 (Bucket → Propriétés → Hébergement de site web statique). Définissez `index.html` comme document d'index et `404.html` comme document d'erreur — DocAnvil génère les deux.

Pour le HTTPS et la mise en cache en périphérie, placez CloudFront devant un bucket privé :

1. Gardez **Bloquer tout accès public** activé sur le bucket
2. Créez une distribution CloudFront avec une politique Origin Access Control (OAC)
3. Faites pointer la distribution vers le bucket et définissez l'objet racine par défaut sur `index.html`
4. Décommentez la ligne `create-invalidation` dans le pipeline afin que les visiteurs voient toujours le contenu actualisé après un déploiement

## Ce qui se passe à chaque push

Quand vous poussez sur `master` :

1. Bitbucket Pipelines récupère `atlassian/default-image:4`
2. Le pipeline télécharge le dernier binaire DocAnvil depuis les releases GitHub — aucune version figée à maintenir
3. `docanvil build --strict --clean` génère un site statique dans `docs/dist/`
4. `aws s3 sync` envoie uniquement les fichiers modifiés et supprime ceux qui ont été effacés
5. (Optionnellement) CloudFront invalide le cache de périphérie pour que le nouveau contenu soit en ligne en quelques secondes

:::note
Le téléchargement du binaire récupère la dernière version stable à chaque exécution. Si vous avez besoin de builds reproductibles avec une version figée, remplacez la résolution dynamique de version par un tag en dur : `LATEST=v1.2.0`.
:::
