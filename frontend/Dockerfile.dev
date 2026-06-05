FROM node:22

WORKDIR /app

# Copy dependency files first for caching
COPY package.json package-lock.json* ./

RUN npm install

COPY . .

EXPOSE 5173

CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
